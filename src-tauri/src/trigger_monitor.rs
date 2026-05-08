// Detects transient "meeting-like" window states:
//   - PowerPoint slideshow (class `screenClass`)
//   - Any foreground window covering an entire monitor (fullscreen)
//
// Emits `trigger-changed` event whenever the combined "is a meeting-ish
// window currently dominant?" answer flips. Polls every 2s via a timer
// thread — good enough for UX, no need for event hooks here.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::state::AppState;

pub const EVENT_TRIGGER_CHANGED: &str = "trigger-changed";

pub static PPT_ACTIVE: AtomicBool = AtomicBool::new(false);
pub static FULLSCREEN_ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
pub fn spawn(app: AppHandle, state: Arc<AppState>) {
    thread::Builder::new()
        .name("sumgim-trigger-monitor".into())
        .spawn(move || run(app, state))
        .expect("failed to spawn trigger-monitor thread");
}

#[cfg(not(windows))]
pub fn spawn(_app: AppHandle, _state: Arc<AppState>) {}

#[cfg(windows)]
fn run(app: AppHandle, state: Arc<AppState>) {
    let mut last_ppt = false;
    let mut last_fullscreen = false;
    loop {
        thread::sleep(Duration::from_secs(2));

        let settings = state.settings.read().clone();
        if !settings.detect_ppt && !settings.detect_fullscreen {
            continue;
        }

        let ppt = if settings.detect_ppt { is_ppt_slideshow() } else { false };
        let fullscreen = if settings.detect_fullscreen { is_fullscreen() } else { false };

        if ppt != last_ppt || fullscreen != last_fullscreen {
            last_ppt = ppt;
            last_fullscreen = fullscreen;
            PPT_ACTIVE.store(ppt, Ordering::Relaxed);
            FULLSCREEN_ACTIVE.store(fullscreen, Ordering::Relaxed);
            let active = ppt || fullscreen;
            let _ = app.emit(EVENT_TRIGGER_CHANGED, active);
        }
    }
}

#[cfg(windows)]
fn is_ppt_slideshow() -> bool {
    use std::cell::Cell;
    use windows::core::BOOL;
    use windows::Win32::Foundation::{HWND, LPARAM, FALSE, TRUE};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, IsWindowVisible,
    };

    thread_local! {
        static FOUND: Cell<bool> = const { Cell::new(false) };
    }
    FOUND.with(|f| f.set(false));

    unsafe extern "system" fn cb(hwnd: HWND, _lp: LPARAM) -> BOOL {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() {
                return TRUE;
            }
            let mut buf = [0u16; 64];
            let n = GetClassNameW(hwnd, &mut buf);
            if n <= 0 {
                return TRUE;
            }
            let class = String::from_utf16_lossy(&buf[..n as usize]);
            if class == "screenClass" {
                FOUND.with(|f| f.set(true));
                return FALSE;
            }
        }
        TRUE
    }

    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(0));
    }
    FOUND.with(|f| f.get())
}

#[cfg(windows)]
fn is_fullscreen() -> bool {
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowRect,
    };

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }

        let mut buf = [0u16; 64];
        let n = GetClassNameW(hwnd, &mut buf);
        if n > 0 {
            let class = String::from_utf16_lossy(&buf[..n as usize]);
            let skip = matches!(
                class.as_str(),
                "Shell_TrayWnd" | "Progman" | "WorkerW" | "Windows.UI.Core.CoreWindow"
            );
            if skip {
                return false;
            }
        }

        let mut wr = RECT::default();
        if GetWindowRect(hwnd, &mut wr).is_err() {
            return false;
        }
        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if hmon.0.is_null() {
            return false;
        }
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(hmon, &mut info).as_bool() {
            return false;
        }
        let mr = info.rcMonitor;
        wr.left == mr.left
            && wr.top == mr.top
            && wr.right == mr.right
            && wr.bottom == mr.bottom
    }
}
