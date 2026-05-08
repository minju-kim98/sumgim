// Global shortcut registration. We keep track of what's currently registered
// so we can unregister + rebind when the user changes the shortcut from Settings.

use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::commands;
use crate::state::{AppState, MeetingSource};

/// Translate a user-visible shortcut string like "Ctrl+Alt+M" into the
/// accelerator syntax used by `tauri-plugin-global-shortcut` (which wants
/// plain "CommandOrControl", "Alt", "Shift", "Super" separated by "+").
fn normalize(shortcut: &str) -> String {
    shortcut
        .split('+')
        .map(|p| match p.trim() {
            "Ctrl" | "Control" => "CommandOrControl".to_string(),
            "Cmd" | "Command" => "CommandOrControl".to_string(),
            "Win" | "Meta" | "Super" => "Super".to_string(),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join("+")
}

pub fn register_initial(app: &AppHandle, state: &Arc<AppState>) -> Result<()> {
    let shortcut = state.settings.read().shortcut.clone();
    rebind(app, state, &shortcut)
}

pub fn rebind(app: &AppHandle, state: &Arc<AppState>, shortcut: &str) -> Result<()> {
    let normalized = normalize(shortcut);
    let parsed =
        Shortcut::from_str(&normalized).context("invalid shortcut accelerator string")?;

    let gs = app.global_shortcut();

    // Unregister old binding if we had one.
    if let Some(old) = state.active_shortcut.write().take() {
        let old_norm = normalize(&old);
        if let Ok(old_parsed) = Shortcut::from_str(&old_norm) {
            let _ = gs.unregister(old_parsed);
        }
    }

    gs.register(parsed)
        .with_context(|| format!("failed to register shortcut '{shortcut}'"))?;

    *state.active_shortcut.write() = Some(shortcut.to_string());
    Ok(())
}

/// Called from lib.rs when building the plugin: delivers the "toggle meeting"
/// action whenever ANY registered shortcut fires. We only have one shortcut at a
/// time so this is fine.
pub fn handle_shortcut_event(
    app: &AppHandle,
    state: &Arc<AppState>,
    event_state: ShortcutState,
) {
    if event_state != ShortcutState::Pressed {
        return;
    }
    let next_active = !state.meeting.read().active;
    if let Err(e) = commands::apply_meeting(app, state, next_active, MeetingSource::Manual) {
        eprintln!("shortcut toggle failed: {e:#}");
    }
}
