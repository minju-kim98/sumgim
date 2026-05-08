use serde::Deserialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;

use crate::dnd;
use crate::kakao_suspend;
use crate::messenger::{self, MessengerCreds};
use crate::shortcut;
use crate::state::{
    AppSettings, AppState, MeetingSource, MeetingState, KEY_AUTOSTART, KEY_AUTO_DETECT_CLONE,
    KEY_DETECT_FULLSCREEN, KEY_DETECT_PPT, KEY_MATTERMOST_ENABLED, KEY_MATTERMOST_TOKEN,
    KEY_MATTERMOST_URL, KEY_MEETING_ACTIVE, KEY_MM_STATUS_EMOJI, KEY_MM_STATUS_TEXT,
    KEY_SHORTCUT, KEY_SHOW_FLOATING, KEY_SLACK_ENABLED, KEY_SLACK_STATUS_EMOJI,
    KEY_SLACK_STATUS_TEXT, KEY_SLACK_TOKEN, KEY_SUSPEND_KAKAOTALK, KEY_TIMEOUT_MINUTES,
    KEY_TOAST_ENABLED_BACKUP, STORE_FILE,
};

/// Map any `anyhow::Error` into a String so Tauri commands can return it as a
/// plain error to JavaScript.
fn to_string_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[derive(Debug, Deserialize, Default)]
pub struct SettingsPatch {
    pub shortcut: Option<String>,
    pub show_floating: Option<bool>,
    pub auto_detect_clone: Option<bool>,
    pub timeout_minutes: Option<u32>,
    pub suspend_kakaotalk: Option<bool>,
    pub detect_ppt: Option<bool>,
    pub detect_fullscreen: Option<bool>,
    pub mattermost_enabled: Option<bool>,
    pub slack_enabled: Option<bool>,
}

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> AppSettings {
    state.settings.read().clone()
}

#[tauri::command]
pub fn get_meeting_state(state: State<'_, Arc<AppState>>) -> MeetingState {
    state.meeting.read().clone()
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    patch: SettingsPatch,
) -> Result<AppSettings, String> {
    {
        let mut s = state.settings.write();
        if let Some(v) = patch.shortcut {
            s.shortcut = v;
        }
        if let Some(v) = patch.show_floating {
            s.show_floating = v;
        }
        if let Some(v) = patch.auto_detect_clone {
            s.auto_detect_clone = v;
        }
        if let Some(v) = patch.timeout_minutes {
            s.timeout_minutes = v.clamp(1, 24 * 60);
        }
        if let Some(v) = patch.suspend_kakaotalk {
            s.suspend_kakaotalk = v;
        }
        if let Some(v) = patch.detect_ppt {
            s.detect_ppt = v;
        }
        if let Some(v) = patch.detect_fullscreen {
            s.detect_fullscreen = v;
        }
        if let Some(v) = patch.mattermost_enabled {
            s.mattermost_enabled = v;
        }
        if let Some(v) = patch.slack_enabled {
            s.slack_enabled = v;
        }
    }
    persist_settings(&app, &state).map_err(to_string_err)?;
    Ok(state.settings.read().clone())
}

#[tauri::command]
pub fn get_messenger_creds(state: State<'_, Arc<AppState>>) -> MessengerCreds {
    state.messenger_creds.read().clone()
}

#[tauri::command]
pub fn update_messenger_creds(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    creds: MessengerCreds,
) -> Result<(), String> {
    *state.messenger_creds.write() = creds;
    persist_settings(&app, &state).map_err(to_string_err)?;
    Ok(())
}

#[tauri::command]
pub fn test_mattermost_connection(url: String, token: String) -> Result<String, String> {
    messenger::test_mattermost(&url, &token).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn test_slack_connection(token: String) -> Result<String, String> {
    messenger::test_slack(&token).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_meeting(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    source: MeetingSource,
) -> Result<MeetingState, String> {
    let next_active = !state.meeting.read().active;
    apply_meeting(&app, &state, next_active, source).map_err(to_string_err)?;
    Ok(state.meeting.read().clone())
}

#[tauri::command]
pub fn set_meeting(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    active: bool,
    source: MeetingSource,
) -> Result<MeetingState, String> {
    apply_meeting(&app, &state, active, source).map_err(to_string_err)?;
    Ok(state.meeting.read().clone())
}

#[tauri::command]
pub fn set_shortcut(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    shortcut: String,
) -> Result<(), String> {
    shortcut::rebind(&app, &state, &shortcut).map_err(to_string_err)?;
    {
        let mut s = state.settings.write();
        s.shortcut = shortcut;
    }
    persist_settings(&app, &state).map_err(to_string_err)?;
    Ok(())
}

#[tauri::command]
pub fn set_autostart(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<(), String> {
    let autolaunch = app.autolaunch();
    if enabled {
        autolaunch.enable().map_err(to_string_err)?;
    } else {
        autolaunch.disable().map_err(to_string_err)?;
    }
    {
        let mut s = state.settings.write();
        s.autostart = enabled;
    }
    persist_settings(&app, &state).map_err(to_string_err)?;
    Ok(())
}

#[tauri::command]
pub fn set_floating_visible(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    visible: bool,
) -> Result<(), String> {
    apply_floating_visibility(&app, visible).map_err(to_string_err)?;
    {
        let mut s = state.settings.write();
        s.show_floating = visible;
    }
    persist_settings(&app, &state).map_err(to_string_err)?;
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

// ───────────────── helpers reused from other modules ─────────────────

/// Central place to flip meeting mode: handles DND side-effects, state bookkeeping,
/// persistence, tray menu refresh, and a "meeting-changed" event broadcast.
pub fn apply_meeting(
    app: &AppHandle,
    state: &Arc<AppState>,
    active: bool,
    source: MeetingSource,
) -> anyhow::Result<()> {
    let was_active = state.meeting.read().active;
    if was_active == active {
        return Ok(());
    }

    if active {
        let backup = dnd::enable_dnd()?;
        *state.toast_enabled_backup.write() = Some(backup);
        state.meeting.write().activate(source);
    } else {
        let previous = *state.toast_enabled_backup.read();
        dnd::disable_dnd(previous)?;
        state.meeting.write().deactivate();
    }

    if active {
        if state.settings.read().suspend_kakaotalk {
            if let Err(e) = kakao_suspend::suspend() {
                eprintln!("kakao suspend failed: {e:#}");
            }
        }
    } else {
        // Always attempt resume — even if setting was just turned off, we may
        // have suspended on a previous ON and need to undo.
        if let Err(e) = kakao_suspend::resume() {
            eprintln!("kakao resume failed: {e:#}");
        }
    }

    if active {
        let (creds, mm_on, sl_on) = {
            let s = state.settings.read();
            (
                state.messenger_creds.read().clone(),
                s.mattermost_enabled,
                s.slack_enabled,
            )
        };
        if mm_on || sl_on {
            let snap = crate::messenger::snapshot_unread(&creds);
            *state.unread_baseline.write() = Some(snap);
        } else {
            *state.unread_baseline.write() = None;
        }
        if mm_on {
            let prev = crate::messenger::capture_mm_status(&creds);
            state.status_backup.write().mattermost = prev;
            if let Err(e) = crate::messenger::enable_dnd_mattermost_only(&creds) {
                eprintln!("mattermost DND enable failed: {e:#}");
            }
            if let Err(e) = crate::messenger::apply_mm_status(&creds) {
                eprintln!("mattermost custom status apply failed: {e:#}");
            }
        }
        if sl_on {
            let prev = crate::messenger::capture_slack_status(&creds);
            state.status_backup.write().slack = prev;
            if let Err(e) = crate::messenger::enable_dnd_slack_only(&creds) {
                eprintln!("slack DND enable failed: {e:#}");
            }
            if let Err(e) = crate::messenger::apply_slack_status(&creds) {
                eprintln!("slack custom status apply failed: {e:#}");
            }
        }
    } else {
        let (creds, mm_on, sl_on) = {
            let s = state.settings.read();
            (
                state.messenger_creds.read().clone(),
                s.mattermost_enabled,
                s.slack_enabled,
            )
        };
        let mut backup = state.status_backup.write();
        let mm_backup = backup.mattermost.take();
        let sl_backup = backup.slack.take();
        drop(backup);
        if mm_on {
            if let Err(e) = crate::messenger::clear_mm_status(&creds, mm_backup.as_ref()) {
                eprintln!("mattermost custom status clear failed: {e:#}");
            }
            if let Err(e) = crate::messenger::disable_dnd_mattermost_only(&creds) {
                eprintln!("mattermost DND disable failed: {e:#}");
            }
        }
        if sl_on {
            if let Err(e) = crate::messenger::clear_slack_status(&creds, sl_backup.as_ref()) {
                eprintln!("slack custom status clear failed: {e:#}");
            }
            if let Err(e) = crate::messenger::disable_dnd_slack_only(&creds) {
                eprintln!("slack DND disable failed: {e:#}");
            }
        }
        let baseline = state.unread_baseline.write().take();
        if let Some(base) = baseline {
            let current = crate::messenger::snapshot_unread(&creds);
            let summary = build_missed_summary(&base, &current);
            if !summary.is_empty() {
                emit_missed_toast(app, &summary);
            }
        }
    }

    crate::window_hider::set_meeting_active(active);

    persist_settings(app, state)?;
    refresh_tray(app);

    // Broadcast to any listening webview (Settings, FloatingToggle, …).
    let payload = state.meeting.read().clone();
    let _ = app.emit("meeting-changed", payload);

    Ok(())
}

pub fn apply_floating_visibility(app: &AppHandle, visible: bool) -> anyhow::Result<()> {
    if let Some(win) = app.get_webview_window("floating") {
        if visible {
            win.show()?;
            // Keep it at the top-right corner by default; user can drag from there.
            let _ = position_floating_default(&win);
        } else {
            win.hide()?;
        }
    }
    Ok(())
}

fn position_floating_default(win: &WebviewWindow) -> anyhow::Result<()> {
    use tauri::{LogicalPosition, Position};
    // Put in the top-right on first show; after that the window-state plugin
    // remembers the user's drag position.
    if let Ok(monitor) = win.current_monitor() {
        if let Some(m) = monitor {
            let size = m.size();
            let scale = m.scale_factor();
            let logical_w = size.width as f64 / scale;
            let x = logical_w - 80.0;
            let y = 80.0;
            win.set_position(Position::Logical(LogicalPosition::new(x, y)))?;
        }
    }
    Ok(())
}

fn refresh_tray(app: &AppHandle) {
    // Defer to tray module so it can keep the menu + tooltip in sync.
    crate::tray::refresh(app);
}

fn build_missed_summary(
    base: &crate::messenger::UnreadSnapshot,
    current: &crate::messenger::UnreadSnapshot,
) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    if let (Some(b), Some(c)) = (base.mattermost, current.mattermost) {
        let diff = c.saturating_sub(b);
        if diff > 0 {
            out.push(("Mattermost".into(), diff));
        }
    }
    if let (Some(b), Some(c)) = (base.slack, current.slack) {
        let diff = c.saturating_sub(b);
        if diff > 0 {
            out.push(("Slack".into(), diff));
        }
    }
    out
}

fn emit_missed_toast(app: &tauri::AppHandle, items: &[(String, u64)]) {
    use tauri_plugin_notification::NotificationExt;
    let body = items
        .iter()
        .map(|(name, n)| format!("{name}: {n}개"))
        .collect::<Vec<_>>()
        .join(" · ");
    let _ = app
        .notification()
        .builder()
        .title("회의 끝! 놓친 알림")
        .body(&body)
        .show();
}

/// Write the current settings + meeting_active flag + toast backup into the
/// plugin-store file. Called after any state mutation that should survive restart.
pub fn persist_settings(app: &AppHandle, state: &Arc<AppState>) -> anyhow::Result<()> {
    let store = app.store(STORE_FILE)?;
    let settings = state.settings.read().clone();
    let meeting_active = state.meeting.read().active;
    let backup = *state.toast_enabled_backup.read();

    store.set(KEY_SHORTCUT, serde_json::json!(settings.shortcut));
    store.set(KEY_SHOW_FLOATING, serde_json::json!(settings.show_floating));
    store.set(
        KEY_AUTO_DETECT_CLONE,
        serde_json::json!(settings.auto_detect_clone),
    );
    store.set(
        KEY_TIMEOUT_MINUTES,
        serde_json::json!(settings.timeout_minutes),
    );
    store.set(
        KEY_SUSPEND_KAKAOTALK,
        serde_json::json!(settings.suspend_kakaotalk),
    );
    store.set(KEY_DETECT_PPT, serde_json::json!(settings.detect_ppt));
    store.set(
        KEY_DETECT_FULLSCREEN,
        serde_json::json!(settings.detect_fullscreen),
    );
    store.set(
        KEY_MATTERMOST_ENABLED,
        serde_json::json!(settings.mattermost_enabled),
    );
    store.set(
        KEY_SLACK_ENABLED,
        serde_json::json!(settings.slack_enabled),
    );
    store.set(KEY_AUTOSTART, serde_json::json!(settings.autostart));
    let creds = state.messenger_creds.read().clone();
    store.set(KEY_MATTERMOST_URL, serde_json::json!(creds.mattermost_url));
    store.set(
        KEY_MATTERMOST_TOKEN,
        serde_json::json!(creds.mattermost_token),
    );
    store.set(KEY_SLACK_TOKEN, serde_json::json!(creds.slack_token));
    store.set(
        KEY_MM_STATUS_EMOJI,
        serde_json::json!(creds.mattermost_status_emoji),
    );
    store.set(
        KEY_MM_STATUS_TEXT,
        serde_json::json!(creds.mattermost_status_text),
    );
    store.set(
        KEY_SLACK_STATUS_EMOJI,
        serde_json::json!(creds.slack_status_emoji),
    );
    store.set(
        KEY_SLACK_STATUS_TEXT,
        serde_json::json!(creds.slack_status_text),
    );
    store.set(KEY_MEETING_ACTIVE, serde_json::json!(meeting_active));
    match backup {
        Some(v) => store.set(KEY_TOAST_ENABLED_BACKUP, serde_json::json!(v)),
        None => {
            store.delete(KEY_TOAST_ENABLED_BACKUP);
        }
    };
    store.save()?;
    Ok(())
}

/// Hydrate state from the plugin-store file on startup.
pub fn load_settings(app: &AppHandle, state: &Arc<AppState>) -> anyhow::Result<bool> {
    let store = app.store(STORE_FILE)?;

    let mut s = state.settings.write();
    if let Some(v) = store.get(KEY_SHORTCUT) {
        if let Some(v) = v.as_str() {
            s.shortcut = v.to_string();
        }
    }
    if let Some(v) = store.get(KEY_SHOW_FLOATING) {
        if let Some(v) = v.as_bool() {
            s.show_floating = v;
        }
    }
    if let Some(v) = store.get(KEY_AUTO_DETECT_CLONE) {
        if let Some(v) = v.as_bool() {
            s.auto_detect_clone = v;
        }
    }
    if let Some(v) = store.get(KEY_TIMEOUT_MINUTES) {
        if let Some(v) = v.as_u64() {
            s.timeout_minutes = v.min(24 * 60) as u32;
        }
    }
    if let Some(v) = store.get(KEY_SUSPEND_KAKAOTALK) {
        if let Some(v) = v.as_bool() {
            s.suspend_kakaotalk = v;
        }
    }
    if let Some(v) = store.get(KEY_DETECT_PPT) {
        if let Some(v) = v.as_bool() {
            s.detect_ppt = v;
        }
    }
    if let Some(v) = store.get(KEY_DETECT_FULLSCREEN) {
        if let Some(v) = v.as_bool() {
            s.detect_fullscreen = v;
        }
    }
    if let Some(v) = store.get(KEY_MATTERMOST_ENABLED) {
        if let Some(v) = v.as_bool() {
            s.mattermost_enabled = v;
        }
    }
    if let Some(v) = store.get(KEY_SLACK_ENABLED) {
        if let Some(v) = v.as_bool() {
            s.slack_enabled = v;
        }
    }
    if let Some(v) = store.get(KEY_AUTOSTART) {
        if let Some(v) = v.as_bool() {
            s.autostart = v;
        }
    }
    let crash_recovery = store
        .get(KEY_MEETING_ACTIVE)
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    s.meeting_active = crash_recovery;
    drop(s);

    if let Some(v) = store.get(KEY_TOAST_ENABLED_BACKUP) {
        if let Some(v) = v.as_u64() {
            *state.toast_enabled_backup.write() = Some(v as u32);
        }
    }

    {
        let mut creds = state.messenger_creds.write();
        if let Some(v) = store.get(KEY_MATTERMOST_URL) {
            if let Some(v) = v.as_str() {
                creds.mattermost_url = v.to_string();
            }
        }
        if let Some(v) = store.get(KEY_MATTERMOST_TOKEN) {
            if let Some(v) = v.as_str() {
                creds.mattermost_token = v.to_string();
            }
        }
        if let Some(v) = store.get(KEY_SLACK_TOKEN) {
            if let Some(v) = v.as_str() {
                creds.slack_token = v.to_string();
            }
        }
        if let Some(v) = store.get(KEY_MM_STATUS_EMOJI) {
            if let Some(v) = v.as_str() {
                creds.mattermost_status_emoji = v.to_string();
            }
        }
        if let Some(v) = store.get(KEY_MM_STATUS_TEXT) {
            if let Some(v) = v.as_str() {
                creds.mattermost_status_text = v.to_string();
            }
        }
        if let Some(v) = store.get(KEY_SLACK_STATUS_EMOJI) {
            if let Some(v) = v.as_str() {
                creds.slack_status_emoji = v.to_string();
            }
        }
        if let Some(v) = store.get(KEY_SLACK_STATUS_TEXT) {
            if let Some(v) = v.as_str() {
                creds.slack_status_text = v.to_string();
            }
        }
    }

    Ok(crash_recovery)
}
