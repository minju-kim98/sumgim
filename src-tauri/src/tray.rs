use std::sync::Arc;

use anyhow::Result;
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::commands;
use crate::state::{AppState, MeetingSource};

const TRAY_ICON_BYTES: &[u8] = include_bytes!("../icons/tray.png");
const TRAY_ID: &str = "sumgim-tray";

// Menu item IDs.
const ID_TOGGLE: &str = "toggle_meeting";
const ID_OPEN_SETTINGS: &str = "open_settings";
const ID_QUIT: &str = "quit";

pub fn build(app: &AppHandle, _state: &Arc<AppState>) -> Result<TrayIcon> {
    let menu = build_menu(app, false)?;
    let icon = Image::from_bytes(TRAY_ICON_BYTES)?;

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(false)
        .tooltip("숨김 — 회의 모드 OFF")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| handle_menu_event(app, event.id.as_ref()))
        .on_tray_icon_event(move |tray, event| handle_tray_event(tray, event))
        .build(app)?;

    Ok(tray)
}

fn build_menu(app: &AppHandle, active: bool) -> Result<Menu<tauri::Wry>> {
    let toggle = CheckMenuItem::with_id(
        app,
        ID_TOGGLE,
        "회의 모드",
        true,
        active,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(app, ID_OPEN_SETTINGS, "설정 열기", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, ID_QUIT, "종료", true, None::<&str>)?;

    Menu::with_items(app, &[&toggle, &settings, &sep, &quit]).map_err(Into::into)
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        ID_TOGGLE => {
            let state: tauri::State<'_, Arc<AppState>> = app.state();
            let next = !state.meeting.read().active;
            if let Err(e) = commands::apply_meeting(app, state.inner(), next, MeetingSource::Manual)
            {
                eprintln!("tray toggle failed: {e:#}");
            }
        }
        ID_OPEN_SETTINGS => {
            show_settings(app);
        }
        ID_QUIT => {
            app.exit(0);
        }
        _ => {}
    }
}

fn handle_tray_event(tray: &TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        let app = tray.app_handle();
        let state: tauri::State<'_, Arc<AppState>> = app.state();
        let next = !state.meeting.read().active;
        if let Err(e) = commands::apply_meeting(app, state.inner(), next, MeetingSource::Manual) {
            eprintln!("tray click toggle failed: {e:#}");
        }
    }
}

pub fn show_settings(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

/// Refresh tooltip + menu check-state to match the current meeting state.
pub fn refresh(app: &AppHandle) {
    let state: tauri::State<'_, Arc<AppState>> = app.state();
    let active = state.meeting.read().active;

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = if active {
            "숨김 — 회의 모드 ON"
        } else {
            "숨김 — 회의 모드 OFF"
        };
        let _ = tray.set_tooltip(Some(tooltip));

        if let Ok(menu) = build_menu(app, active) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
