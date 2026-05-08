// Popup-window blocker for apps that use custom topmost windows (KakaoTalk, etc.)
// instead of Windows toast notifications.
//
// Uses SetWinEventHook(EVENT_OBJECT_SHOW, WINEVENT_OUTOFCONTEXT) to get
// cross-process "a window just got shown" events, runs a message loop on a
// dedicated thread, and hides windows that match a configurable blacklist.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use tauri::AppHandle;

pub static MEETING_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn set_meeting_active(active: bool) {
    MEETING_ACTIVE.store(active, Ordering::Relaxed);
}

#[cfg(windows)]
pub fn spawn(_app: AppHandle) {
    thread::Builder::new()
        .name("sumgim-window-hider".into())
        .spawn(run)
        .expect("failed to spawn window-hider thread");
}

#[cfg(not(windows))]
pub fn spawn(_app: AppHandle) {}

#[cfg(windows)]
fn run() {
    use windows::Win32::UI::Accessibility::SetWinEventHook;
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, EVENT_OBJECT_SHOW,
        EVENT_OBJECT_UNCLOAKED, MSG, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
    };

    unsafe {
        // Register a range so we also catch DWM "uncloak" — some modern apps
        // (Electron, WinUI, and possibly KakaoTalk notifications) never emit
        // EVENT_OBJECT_SHOW; they reveal a pre-cloaked window instead.
        let _hook = SetWinEventHook(
            EVENT_OBJECT_SHOW,
            EVENT_OBJECT_UNCLOAKED,
            None,
            Some(on_event),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(windows)]
unsafe extern "system" fn on_event(
    _hook: windows::Win32::UI::Accessibility::HWINEVENTHOOK,
    event: u32,
    hwnd: windows::Win32::Foundation::HWND,
    id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    use windows::Win32::UI::WindowsAndMessaging::{
        EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED, OBJID_WINDOW,
    };

    if event != EVENT_OBJECT_SHOW && event != EVENT_OBJECT_UNCLOAKED {
        return;
    }
    if id_object != OBJID_WINDOW.0 {
        return;
    }
    if !MEETING_ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    if hwnd.0.is_null() {
        return;
    }
    if should_hide(hwnd, event) {
        hide(hwnd);
    }
}

#[cfg(windows)]
fn should_hide(hwnd: windows::Win32::Foundation::HWND, event: u32) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, IsWindowVisible};

    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return false;
        }

        let mut buf = [0u16; 256];
        let n = GetClassNameW(hwnd, &mut buf);
        if n <= 0 {
            return false;
        }
        let class = String::from_utf16_lossy(&buf[..n as usize]);

        let proc_name = process_name_of(hwnd).unwrap_or_default();
        let lower = proc_name.to_lowercase();
        if lower != "kakaotalk.exe" {
            return false;
        }

        let (w, h) = window_size(hwnd);
        let evt = if event == 0x8002 { "SHOW" } else { "UNCLOAK" };

        // Heuristic: the main app window is big (>= 400w AND >= 500h). Everything
        // smaller is a popup / notification / profile card → hide. This is more
        // robust than matching a specific class name, which varies by version.
        let is_main = w >= 400 && h >= 500;

        log(&format!(
            "kakao {evt}: class='{class}' size={w}x{h} -> {}",
            if is_main { "skip (main)" } else { "hide" }
        ));

        !is_main
    }
}

#[cfg(windows)]
fn window_size(hwnd: windows::Win32::Foundation::HWND) -> (i32, i32) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
    unsafe {
        let mut r = RECT::default();
        if GetWindowRect(hwnd, &mut r).is_err() {
            return (0, 0);
        }
        (r.right - r.left, r.bottom - r.top)
    }
}

#[cfg(windows)]
fn log(msg: &str) {
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
        let _ = writeln!(f, "[{secs}] {msg}");
    }
}

#[cfg(windows)]
fn process_name_of(hwnd: windows::Win32::Foundation::HWND) -> Option<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        )
        .ok()?;

        let mut buf = [0u16; 260];
        let n = GetModuleBaseNameW(handle, None, &mut buf);
        let _ = CloseHandle(handle);
        if n == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..n as usize]))
    }
}

#[cfg(windows)]
fn hide(hwnd: windows::Win32::Foundation::HWND) {
    // Use DWM cloak instead of ShowWindow(SW_HIDE). Apps like KakaoTalk animate
    // their popups by repeatedly calling ShowWindow — fighting that with SW_HIDE
    // creates a flicker loop. DWM cloak is layered above ShowWindow state and
    // cannot be overridden by the app, so the cloak sticks for the popup's whole
    // lifetime. Same mechanism Windows uses to hide windows from Alt+Tab.
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_CLOAK};
    unsafe {
        let cloak: i32 = 1; // TRUE
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CLOAK,
            &cloak as *const _ as *const _,
            std::mem::size_of::<i32>() as u32,
        );
    }
}
