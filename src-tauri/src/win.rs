//! Windows-only protections: no screen capture, a private clipboard, and locking when Windows
//! locks or goes to sleep.

use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{GlobalFree, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardSequenceNumber, OpenClipboard, RegisterClipboardFormatW,
    SetClipboardData,
};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::System::RemoteDesktop::WTSRegisterSessionNotification;
use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::SetWindowDisplayAffinity;
use zeroize::Zeroize;

const WDA_EXCLUDEFROMCAPTURE: u32 = 0x11;
const CF_UNICODETEXT: u32 = 13;
const NOTIFY_FOR_THIS_SESSION: u32 = 0;
const WM_WTSSESSION_CHANGE: u32 = 0x02B1;
const WTS_SESSION_LOCK: usize = 0x7;
const WM_POWERBROADCAST: u32 = 0x0218;
const PBT_APMSUSPEND: usize = 0x4;
const SUBCLASS_ID: usize = 0x4B50; // "KP"

/// Makes the window show as black in screenshots, screen recordings, screen sharing and Recall.
pub fn block_capture(hwnd: HWND) {
    unsafe {
        SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
    }
}

static ON_LOCK: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

unsafe extern "system" fn session_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    let lock = (msg == WM_WTSSESSION_CHANGE && wparam == WTS_SESSION_LOCK)
        || (msg == WM_POWERBROADCAST && wparam == PBT_APMSUSPEND);
    if lock {
        if let Some(f) = ON_LOCK.get() {
            f();
        }
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
}

/// Calls `on_lock` when Windows locks (Win+L, lid close) or is about to sleep or hibernate.
/// Must run on the thread that owns the window.
pub fn watch_session(hwnd: HWND, on_lock: impl Fn() + Send + Sync + 'static) {
    if ON_LOCK.set(Box::new(on_lock)).is_err() {
        return;
    }
    unsafe {
        WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);
        SetWindowSubclass(hwnd, Some(session_proc), SUBCLASS_ID, 0);
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe fn put(format: u32, bytes: &[u8]) -> bool {
    let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1));
    if h.is_null() {
        return false;
    }
    let p = GlobalLock(h) as *mut u8;
    if p.is_null() {
        GlobalFree(h);
        return false;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
    GlobalUnlock(h);
    if SetClipboardData(format, h).is_null() {
        GlobalFree(h);
        return false;
    }
    true
}

unsafe fn open_clipboard() -> bool {
    // Another app may hold the clipboard for a moment.
    for _ in 0..10 {
        if OpenClipboard(std::ptr::null_mut()) != 0 {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
    false
}

/// Copies text and marks it so it stays out of Win+V clipboard history and the cloud clipboard.
/// Returns the clipboard sequence number, to clear it later only if nothing else was copied.
pub fn copy_sensitive(text: &str) -> Option<u32> {
    let mut utf16 = wide(text);
    let bytes: &[u8] = unsafe { std::slice::from_raw_parts(utf16.as_ptr() as *const u8, utf16.len() * 2) };
    let result = unsafe {
        if !open_clipboard() {
            None
        } else {
            EmptyClipboard();
            let ok = put(CF_UNICODETEXT, bytes);
            if ok {
                let zero = 0u32.to_ne_bytes();
                for name in ["ExcludeClipboardContentFromMonitorProcessing", "CanIncludeInClipboardHistory", "CanUploadToCloudClipboard"] {
                    let format = RegisterClipboardFormatW(wide(name).as_ptr());
                    if format != 0 {
                        put(format, &zero);
                    }
                }
            }
            CloseClipboard();
            ok.then(|| GetClipboardSequenceNumber())
        }
    };
    utf16.zeroize();
    result
}

/// Empties the clipboard if it still holds what Keyplate put there.
pub fn clear_clipboard_if(sequence: u32) {
    unsafe {
        if GetClipboardSequenceNumber() == sequence && open_clipboard() {
            EmptyClipboard();
            CloseClipboard();
        }
    }
}
