mod commands;
mod display_monitor;
mod dnd;
mod kakao_suspend;
mod messenger;
mod shortcut;
mod state;
mod tray;
mod trigger_monitor;
mod window_hider;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;

use crate::state::{AppState, MeetingSource};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new();
    let shortcut_state = state.clone();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::SIZE,
                )
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    shortcut::handle_shortcut_event(app, &shortcut_state, event.state());
                })
                .build(),
        )
        .manage(state.clone());

    builder
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::get_meeting_state,
            commands::toggle_meeting,
            commands::set_meeting,
            commands::set_shortcut,
            commands::set_autostart,
            commands::set_floating_visible,
            commands::quit_app,
            commands::get_messenger_creds,
            commands::update_messenger_creds,
            commands::test_mattermost_connection,
            commands::test_slack_connection,
            commands::complete_onboarding,
        ])
        .on_window_event(|window, event| {
            // Main window close = hide to tray instead of quit.
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            let state_for_setup = state.clone();
            setup(handle, state_for_setup)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup(app: AppHandle, state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    // 1) Load persisted settings + detect crash recovery.
    let crash_recovered_active = commands::load_settings(&app, &state)?;

    // 2) Build tray + menu.
    tray::build(&app, &state)?;

    // 3) Register the global shortcut (takes value from just-loaded settings).
    if let Err(e) = shortcut::register_initial(&app, &state) {
        eprintln!("could not register initial shortcut: {e:#}");
    }

    // 4) Show the main window only on first launch (onboarding wizard); otherwise launch silent.
    if let Some(win) = app.get_webview_window("main") {
        if !state.settings.read().onboarding_done {
            let _ = win.show();
            let _ = win.unminimize();
            let _ = win.set_focus();
        } else {
            let _ = win.hide();
        }
    }

    // 5) Floating window visibility follows the stored preference.
    let show_floating = state.settings.read().show_floating;
    let _ = commands::apply_floating_visibility(&app, show_floating);

    // 5b) Reconcile autostart registry entry with stored preference.
    let stored_autostart = state.settings.read().autostart;
    let autolaunch = app.autolaunch();
    if let Ok(current) = autolaunch.is_enabled() {
        if stored_autostart && !current {
            let _ = autolaunch.enable();
        } else if !stored_autostart && current {
            let _ = autolaunch.disable();
        }
    }

    // 6) Crash recovery: if we died mid-meeting, release DND so user isn't
    //    stuck muted indefinitely.
    if crash_recovered_active {
        eprintln!("[sumgim] recovering from crash: previous session was in meeting mode, releasing DND");
        let previous = *state.toast_enabled_backup.read();
        if let Err(e) = dnd::disable_dnd(previous) {
            eprintln!("crash-recovery DND release failed: {e:#}");
        }
        if let Err(e) = kakao_suspend::resume() {
            eprintln!("crash-recovery kakao resume failed: {e:#}");
        }
        crate::window_hider::set_meeting_active(false);
        // Reset state + persist the cleared flag.
        state.meeting.write().deactivate();
        let _ = commands::persist_settings(&app, &state);
    }

    // 7) Start display-monitor thread. The thread emits "display-clone-changed"
    //    events; we listen here in the main process and decide whether to
    //    auto-toggle meeting mode.
    display_monitor::spawn(app.clone(), state.clone());

    window_hider::spawn(app.clone());

    let app_for_listener = app.clone();
    let state_for_listener = state.clone();
    app.listen(display_monitor::EVENT_CLONE_CHANGED, move |event| {
        let cloning: bool = serde_json::from_str(event.payload()).unwrap_or(false);
        on_clone_changed(&app_for_listener, &state_for_listener, cloning);
    });

    let app_for_external = app.clone();
    let state_for_external = state.clone();
    app.listen(
        display_monitor::EVENT_EXTERNAL_DISPLAY_ATTACHED,
        move |_event| {
            on_external_display_attached(&app_for_external, &state_for_external);
        },
    );

    trigger_monitor::spawn(app.clone(), state.clone());

    let app_for_trigger = app.clone();
    let state_for_trigger = state.clone();
    app.listen(trigger_monitor::EVENT_TRIGGER_CHANGED, move |event| {
        let active: bool = serde_json::from_str(event.payload()).unwrap_or(false);
        on_trigger_changed(&app_for_trigger, &state_for_trigger, active);
    });

    // 8) Timeout watchdog: a single thread that wakes every 30s and checks
    //    whether an auto-enabled meeting has exceeded the user's timeout.
    let app_for_timeout = app.clone();
    let state_for_timeout = state.clone();
    thread::Builder::new()
        .name("sumgim-timeout".into())
        .spawn(move || timeout_watchdog(app_for_timeout, state_for_timeout))?;

    // 8b) Background updater poll: 앱 시작 30초 후 1회, 이후 24시간마다 체크.
    //     새 버전이 있으면 토스트로 알림. 다운로드/설치는 사용자가 트레이 →
    //     "업데이트 확인" 또는 설정 화면에서 명시적으로 트리거.
    let app_for_updater = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(30)).await;
        loop {
            background_update_check(&app_for_updater).await;
            tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
        }
    });

    // 9) Final tray refresh to reflect post-recovery state.
    tray::refresh(&app);

    Ok(())
}

fn any_auto_trigger_active(state: &Arc<AppState>) -> bool {
    let settings = state.settings.read().clone();
    let cloning = if settings.auto_detect_clone {
        crate::display_monitor::is_cloning()
    } else {
        false
    };
    let ppt = settings.detect_ppt
        && crate::trigger_monitor::PPT_ACTIVE.load(std::sync::atomic::Ordering::Relaxed);
    let fs = settings.detect_fullscreen
        && crate::trigger_monitor::FULLSCREEN_ACTIVE.load(std::sync::atomic::Ordering::Relaxed);
    cloning || ppt || fs
}

fn on_clone_changed(app: &AppHandle, state: &Arc<AppState>, cloning: bool) {
    let settings = state.settings.read().clone();
    let meeting = state.meeting.read().clone();

    display_monitor::log(&format!(
        "on_clone_changed: cloning={cloning}, auto_detect_clone={}, meeting.active={}, source={:?}",
        settings.auto_detect_clone, meeting.active, meeting.source
    ));

    if cloning {
        if settings.auto_detect_clone && !meeting.active {
            display_monitor::log("auto-enabling meeting (clone detected)");
            if let Err(e) = commands::apply_meeting(app, state, true, MeetingSource::Auto) {
                display_monitor::log(&format!("auto-enable failed: {e:#}"));
                eprintln!("auto-enable (clone) failed: {e:#}");
                return;
            }
            toast(app, "숨김: 회의 모드 자동 시작", "디스플레이 복제가 감지되었습니다");
        } else {
            display_monitor::log("skip auto-enable (setting off or meeting already active)");
        }
    } else {
        // Clone released: only auto-disable if WE turned it on AND no other
        // auto trigger is still holding meeting mode open.
        if meeting.active
            && meeting.source == Some(MeetingSource::Auto)
            && !any_auto_trigger_active(state)
        {
            display_monitor::log("auto-disabling meeting (clone released)");
            if let Err(e) = commands::apply_meeting(app, state, false, MeetingSource::Auto) {
                display_monitor::log(&format!("auto-disable failed: {e:#}"));
                eprintln!("auto-disable (clone released) failed: {e:#}");
                return;
            }
            toast(app, "숨김: 회의 모드 자동 해제", "디스플레이 복제가 해제되었습니다");
        }
    }
}

fn on_external_display_attached(app: &AppHandle, state: &Arc<AppState>) {
    let meeting_active = state.meeting.read().active;
    if meeting_active {
        // Already in meeting mode — no need to suggest.
        return;
    }
    let shortcut = state.settings.read().shortcut.clone();
    toast(
        app,
        "숨김: 외부 모니터 연결됨",
        &format!("회의 시작이라면 {shortcut}로 회의 모드를 켜세요."),
    );
}

fn on_trigger_changed(app: &AppHandle, state: &Arc<AppState>, active: bool) {
    let meeting = state.meeting.read().clone();
    let ppt = trigger_monitor::PPT_ACTIVE.load(std::sync::atomic::Ordering::Relaxed);
    let fs = trigger_monitor::FULLSCREEN_ACTIVE.load(std::sync::atomic::Ordering::Relaxed);

    if active {
        if !meeting.active {
            let body = match (ppt, fs) {
                (true, true) => "PowerPoint + 전체화면",
                (true, false) => "PowerPoint 슬라이드쇼가 시작되었습니다",
                (false, true) => "전체 화면 앱이 실행되었습니다",
                _ => "자동 트리거가 감지되었습니다",
            };
            if let Err(e) = commands::apply_meeting(app, state, true, MeetingSource::Auto) {
                eprintln!("auto-enable (trigger) failed: {e:#}");
                return;
            }
            toast(app, "숨김: 회의 모드 자동 시작", body);
        }
    } else {
        if meeting.active
            && meeting.source == Some(MeetingSource::Auto)
            && !any_auto_trigger_active(state)
        {
            if let Err(e) = commands::apply_meeting(app, state, false, MeetingSource::Auto) {
                eprintln!("auto-disable (trigger released) failed: {e:#}");
                return;
            }
            toast(
                app,
                "숨김: 회의 모드 자동 해제",
                "자동 트리거가 해제되었습니다",
            );
        }
    }
}

fn timeout_watchdog(app: AppHandle, state: Arc<AppState>) {
    loop {
        thread::sleep(Duration::from_secs(30));
        let (should_disable, timeout_minutes) = {
            let meeting = state.meeting.read();
            let settings = state.settings.read();
            let should = matches!(
                (meeting.active, meeting.source, meeting.since_epoch_secs),
                (true, Some(MeetingSource::Auto), Some(_))
            ) && {
                let now = state::now_secs();
                let since = meeting.since_epoch_secs.unwrap_or(now);
                now.saturating_sub(since) >= (settings.timeout_minutes as u64) * 60
            };
            (should, settings.timeout_minutes)
        };
        if should_disable {
            if let Err(e) = commands::apply_meeting(&app, &state, false, MeetingSource::Auto) {
                eprintln!("timeout auto-disable failed: {e:#}");
                continue;
            }
            toast(
                &app,
                "숨김: 회의 모드 자동 해제",
                &format!("{timeout_minutes}분 타임아웃이 지났습니다"),
            );
        }
    }
}

async fn background_update_check(app: &AppHandle) {
    use tauri_plugin_updater::UpdaterExt;
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("[sumgim] updater handle init failed: {e:#}");
            return;
        }
    };
    match updater.check().await {
        Ok(Some(update)) => {
            let body = format!(
                "v{} 사용 가능. 트레이 → '업데이트 확인'에서 받을 수 있습니다.",
                update.version
            );
            toast(app, "숨김: 업데이트 가능", &body);
        }
        Ok(None) => {}
        Err(e) => eprintln!("[sumgim] update check failed: {e:#}"),
    }
}

fn toast(app: &AppHandle, title: &str, body: &str) {
    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();
    // Also emit an in-app event so webviews can show custom UI if desired.
    let _ = app.emit(
        "toast",
        serde_json::json!({ "title": title, "body": body }),
    );
}
