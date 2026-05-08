// Kakao Talk process-suspend.
//
// Why not just hide the popup window? KakaoTalk animates its notification
// window by repeatedly calling ShowWindow/SetWindowPos. Any attempt to hide
// it externally (SW_HIDE, DWM cloak, offscreen move) creates a flicker battle
// that the app wins. Suspending every thread in KakaoTalk.exe is the only
// reliable way to stop new popups from appearing.
//
// On resume, messages received during suspension arrive in bulk — which is
// the exact "meeting ended → show backlog" UX the PRD §4.5 describes.

use once_cell::sync::Lazy;
use std::sync::Mutex;

const PROCESS_NAME: &str = "KakaoTalk.exe";

static SUSPENDED_PIDS: Lazy<Mutex<Vec<u32>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[cfg(windows)]
#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtSuspendProcess(handle: windows::Win32::Foundation::HANDLE) -> i32;
    fn NtResumeProcess(handle: windows::Win32::Foundation::HANDLE) -> i32;
}

#[cfg(windows)]
fn find_pids() -> anyhow::Result<Vec<u32>> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let target = PROCESS_NAME.to_lowercase();
    let mut out = Vec::new();

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let end = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..end]);
                if name.to_lowercase() == target {
                    out.push(entry.th32ProcessID);
                }
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }

    Ok(out)
}

#[cfg(windows)]
pub fn suspend() -> anyhow::Result<()> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SUSPEND_RESUME};

    // Always clear the list first — if a previous suspend left stale PIDs,
    // resume() would try to resume processes that are either dead or different.
    SUSPENDED_PIDS.lock().unwrap().clear();

    let pids = find_pids()?;
    let mut suspended = Vec::new();

    unsafe {
        for pid in pids {
            let Ok(handle) = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid) else {
                eprintln!("[kakao_suspend] failed to open pid {pid} for suspend");
                continue;
            };
            let status = NtSuspendProcess(handle);
            let _ = CloseHandle(handle);
            if status == 0 {
                suspended.push(pid);
            } else {
                eprintln!("[kakao_suspend] NtSuspendProcess pid {pid} returned 0x{status:x}");
            }
        }
    }

    *SUSPENDED_PIDS.lock().unwrap() = suspended;
    Ok(())
}

#[cfg(windows)]
pub fn resume() -> anyhow::Result<()> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SUSPEND_RESUME};

    let pids = std::mem::take(&mut *SUSPENDED_PIDS.lock().unwrap());
    unsafe {
        for pid in pids {
            let Ok(handle) = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid) else {
                eprintln!("[kakao_suspend] failed to open pid {pid} for resume (process likely exited)");
                continue;
            };
            let status = NtResumeProcess(handle);
            let _ = CloseHandle(handle);
            if status != 0 {
                eprintln!("[kakao_suspend] NtResumeProcess pid {pid} returned 0x{status:x}");
            }
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn suspend() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub fn resume() -> anyhow::Result<()> {
    Ok(())
}
