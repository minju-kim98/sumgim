// Windows Do-Not-Disturb control.
//
// Strategy:
//   1. Flip HKCU\...\PushNotifications\ToastEnabled to 0 (notification center
//      respects this within a second or two).
//   2. Start "Presentation Mode" via PresentationSettings.exe which also
//      suppresses system notifications + screensaver + sleep.
//
// On release we restore ToastEnabled to the pre-enable value (so users who had
// it off intentionally aren't overridden) and stop presentation mode.

use anyhow::{Context, Result};

#[cfg(windows)]
const TOAST_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\PushNotifications";
#[cfg(windows)]
const TOAST_VALUE: &str = "ToastEnabled";

#[cfg(windows)]
pub fn read_toast_enabled() -> Result<u32> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey(TOAST_KEY) {
        Ok(key) => {
            let v: u32 = key.get_value(TOAST_VALUE).unwrap_or(1);
            Ok(v)
        }
        // If the subkey doesn't exist yet, Windows treats toasts as enabled.
        Err(_) => Ok(1),
    }
}

#[cfg(windows)]
fn write_toast_enabled(value: u32) -> Result<()> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey_with_flags(TOAST_KEY, KEY_SET_VALUE)
        .context("open HKCU PushNotifications key")?;
    key.set_value(TOAST_VALUE, &value)
        .context("write ToastEnabled")?;
    Ok(())
}

#[cfg(windows)]
fn run_presentation(arg: &str) {
    use std::process::Command;
    // PresentationSettings.exe is built into Windows Pro/Enterprise. On Home
    // editions it may be missing — we fail quietly, the registry flip is still
    // the main effect.
    let _ = Command::new("PresentationSettings.exe").arg(arg).spawn();
}

/// Turn DND on. Returns the pre-change ToastEnabled value which the caller
/// should persist for later restoration.
pub fn enable_dnd() -> Result<u32> {
    #[cfg(windows)]
    {
        let previous = read_toast_enabled()?;
        write_toast_enabled(0)?;
        run_presentation("/start");
        Ok(previous)
    }
    #[cfg(not(windows))]
    {
        // No-op on non-Windows so `cargo check` on other platforms still works.
        Ok(1)
    }
}

/// Restore DND state using the previously saved ToastEnabled value.
/// `previous` defaults to 1 (notifications enabled) if we never captured a
/// backup — safer than leaving the user muted.
pub fn disable_dnd(previous: Option<u32>) -> Result<()> {
    #[cfg(windows)]
    {
        let value = previous.unwrap_or(1);
        write_toast_enabled(value)?;
        run_presentation("/stop");
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = previous;
        Ok(())
    }
}
