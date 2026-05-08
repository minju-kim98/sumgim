// Display clone (mirror) detection via a hidden top-level Win32 window.
//
// We spin up a background thread that owns a hidden HWND, pumps messages, and
// listens for WM_DISPLAYCHANGE. The window must be top-level (not message-only)
// because WM_DISPLAYCHANGE is broadcast only to top-level windows. On each
// change we call QueryDisplayConfig and inspect the active paths: two active
// paths with the same `sourceInfo.id` indicate mirroring (a.k.a. clone mode).
//
// The thread emits a Tauri event "display-clone-changed" whose payload is a
// boolean.  The main process listens and decides (based on user settings +
// current meeting state) whether to auto-enable/disable meeting mode.

use std::sync::Arc;
use std::thread;

use tauri::{AppHandle, Emitter};

use crate::state::AppState;

pub const EVENT_CLONE_CHANGED: &str = "display-clone-changed";

/// Report whether the system is in duplicate/clone mode.
///
/// Criterion: `active_path_count > SM_CMONITORS`. Each physical display output
/// is an active path (so a duplicate setup with N monitors still has N active
/// paths). But `SM_CMONITORS` collapses mirrored outputs into one logical
/// monitor — so the path count exceeding the monitor count is the unambiguous
/// signature of cloning, including on setups (like 3-monitor duplicate) where
/// SM_CMONITORS would otherwise be 1 and simpler virtual-vs-primary heuristics
/// fail.
#[cfg(windows)]
pub fn is_cloning() -> bool {
    use windows::Win32::Devices::Display::{
        GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CMONITORS};

    unsafe {
        let monitors = GetSystemMetrics(SM_CMONITORS);

        let mut path_count = 0u32;
        let mut mode_count = 0u32;
        let r =
            GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count);
        if r.is_err() {
            log(&format!(
                "GetDisplayConfigBufferSizes failed (monitors={monitors}) -> treating as not cloning"
            ));
            return false;
        }

        let active_paths = path_count as i32;
        let cloning = monitors >= 1 && active_paths > monitors;
        log(&format!(
            "active_paths={active_paths}, monitors={monitors}, is_cloning={cloning}"
        ));
        cloning
    }
}

#[cfg(not(windows))]
pub fn is_cloning() -> bool {
    false
}

pub fn log(msg: &str) {
    if !cfg!(debug_assertions) {
        return;
    }
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};
    let Ok(appdata) = std::env::var("APPDATA") else {
        return;
    };
    let dir = std::path::PathBuf::from(&appdata).join("Sumgim");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("debug.log");
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > 5 * 1024 * 1024 {
            let _ = std::fs::remove_file(&path);
        }
    }
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "[{secs}] display: {msg}");
    }
}

#[cfg(windows)]
pub fn spawn(app: AppHandle, _state: Arc<AppState>) {
    thread::Builder::new()
        .name("sumgim-display-monitor".into())
        .spawn(move || run_message_loop(app))
        .expect("failed to spawn display-monitor thread");
}

#[cfg(not(windows))]
pub fn spawn(_app: AppHandle, _state: Arc<AppState>) {
    // No-op on non-Windows.
}

#[cfg(windows)]
fn run_message_loop(app: AppHandle) {
    use std::cell::Cell;
    use std::ptr::null_mut;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassExW,
        TranslateMessage, MSG, WM_DISPLAYCHANGE, WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_POPUP,
    };

    thread_local! {
        static APP_HANDLE: Cell<Option<AppHandle>> = const { Cell::new(None) };
        static LAST_CLONE: Cell<bool> = const { Cell::new(false) };
    }

    log("display_monitor thread started");
    APP_HANDLE.with(|h| h.set(Some(app.clone())));
    let initial = is_cloning();
    LAST_CLONE.with(|c| c.set(initial));
    log(&format!("initial clone state = {initial}"));

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_DISPLAYCHANGE {
            log(&format!("WM_DISPLAYCHANGE received (wparam={:?})", wparam.0));
            let current = is_cloning();
            LAST_CLONE.with(|c| {
                let prev = c.get();
                if prev != current {
                    c.set(current);
                    log(&format!("state changed {prev} -> {current}, emitting event"));
                    APP_HANDLE.with(|h| {
                        if let Some(app) = unsafe { &*h.as_ptr() } {
                            match app.emit(EVENT_CLONE_CHANGED, current) {
                                Ok(_) => log("emit OK"),
                                Err(e) => log(&format!("emit FAILED: {e}")),
                            }
                        } else {
                            log("no AppHandle stored, skipping emit");
                        }
                    });
                } else {
                    log(&format!("state unchanged ({current}), no emit"));
                }
            });
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    unsafe {
        let hmodule = GetModuleHandleW(None).expect("GetModuleHandleW");
        // In windows 0.61 WNDCLASSEXW.hInstance is HINSTANCE; HMODULE and
        // HINSTANCE share the same underlying handle value on Win32.
        let hinstance = windows::Win32::Foundation::HINSTANCE(hmodule.0);
        let class_name = w!("SumgimDisplayMonitor");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..Default::default()
        };
        let atom = RegisterClassExW(&wc);
        log(&format!("RegisterClassExW atom={atom}"));

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            PCWSTR(class_name.as_ptr()),
            w!("Sumgim hidden window"),
            WS_POPUP,
            -10000,
            -10000,
            1,
            1,
            None,
            None,
            Some(hinstance),
            Some(null_mut()),
        );
        match &hwnd {
            Ok(h) => log(&format!("CreateWindowExW OK, hwnd={:p}", h.0)),
            Err(e) => log(&format!("CreateWindowExW FAILED: {e}")),
        }

        log("entering message loop");
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        log("message loop exited (thread dying)");
    }
}
