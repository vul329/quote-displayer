use std::sync::mpsc;
use std::sync::OnceLock;

use tauri::{AppHandle, Emitter, Manager};

use crate::db::AppData;

// ── globals ────────────────────────────────────────────────
static APP: OnceLock<AppHandle> = OnceLock::new();
static TX: OnceLock<std::sync::Mutex<mpsc::Sender<Action>>> =
    OnceLock::new();
static HOOK_HANDLE: OnceLock<std::sync::Mutex<Option<isize>>> =
    OnceLock::new();

enum Action {
    Collect,
    ShowQuote,
}

pub fn init(app: &AppHandle) {
    APP.set(app.clone()).ok();

    let (tx, rx) = mpsc::channel::<Action>();
    TX.get_or_init(|| std::sync::Mutex::new(tx));

    // Background worker – avoids blocking the hook past Windows' 300 ms timeout
    std::thread::spawn(move || {
        let app = APP.get().expect("APP not set");
        let mut clipboard = arboard::Clipboard::new().ok();
        while let Ok(action) = rx.recv() {
            match action {
                Action::Collect => {
                    // Simulate Ctrl+C so user doesn't have to press it manually
                    #[cfg(target_os = "windows")]
                    send_ctrl_c();

                    // Brief pause so the clipboard settles
                    std::thread::sleep(std::time::Duration::from_millis(150));

                    let cb = match clipboard.as_mut() {
                        Some(c) => c,
                        None => continue,
                    };
                    let data = match app.try_state::<AppData>() {
                        Some(d) => d,
                        None => continue,
                    };
                    do_collect(data.inner(), cb);
                    let _ = app.emit("quote-collected", ());
                }
                Action::ShowQuote => {
                    crate::popup::show_quote_popup(app);
                }
            }
        }
    });

    #[cfg(target_os = "windows")]
    install_hook_ll();
}

pub fn shutdown() {
    #[cfg(target_os = "windows")]
    {
        if let Some(mtx) = HOOK_HANDLE.get() {
            if let Ok(mut h) = mtx.lock() {
                if let Some(hook) = h.take() {
                    win32::unhook(hook);
                }
            }
        }
    }
}

// ── simulate Ctrl+C (Windows) ──────────────────────────────
#[cfg(target_os = "windows")]
fn send_ctrl_c() {
    const VK_CONTROL: u16 = 0x11;
    const VK_C: u16 = 0x43;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    unsafe {
        // Ctrl down
        win32::keybd_event(VK_CONTROL as u8, 0, 0, 0);
        // C down
        win32::keybd_event(VK_C as u8, 0, 0, 0);
        // C up
        win32::keybd_event(VK_C as u8, 0, KEYEVENTF_KEYUP, 0);
        // Ctrl up
        win32::keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
    }
}

// ── actual collection logic (runs on background thread) ────
fn do_collect(data: &AppData, cb: &mut arboard::Clipboard) {
    let content = match cb.get_text() {
        Ok(t) if !t.trim().is_empty() => t,
        _ => return,
    };
    let mut quotes = match data.quotes.lock() {
        Ok(q) => q,
        Err(_) => return,
    };
    if !quotes.categories.iter().any(|c| c.name == "未整理") {
        quotes.categories.push(crate::models::Category {
            name: "未整理".to_string(),
        });
    }
    let new_id = quotes.quotes.iter().map(|q| q.id).max().unwrap_or(0) + 1;
    quotes.quotes.push(crate::models::Quote {
        id: new_id,
        content,
        author: None,
        category: "未整理".to_string(),
        enabled: true,
        show_count: 0,
        created_at: chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    });
    drop(quotes);
    data.persist_all();
}

// ── hotkey string parser ──────────────────────────────────
mod parser {
    #[cfg(target_os = "windows")]
    pub fn parse_hotkey(s: &str) -> Option<(u32, u32)> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return None;
        }
        let key_str = parts.last()?.trim();
        let vk = key_to_vk(key_str)?;

        let mut mods: u32 = 0;
        for m in parts.iter().take(parts.len() - 1) {
            match m.trim().to_lowercase().as_str() {
                "ctrl" | "control" => mods |= 0x0002,
                "alt" => mods |= 0x0001,
                "shift" => mods |= 0x0004,
                "win" | "cmd" | "super" => mods |= 0x0008,
                _ => {}
            }
        }
        Some((vk, mods))
    }

    #[cfg(target_os = "windows")]
    fn key_to_vk(s: &str) -> Option<u32> {
        if let Some(num) = s.strip_prefix("F").or_else(|| s.strip_prefix("f"))
        {
            if let Ok(n) = num.parse::<u32>() {
                if (1..=24).contains(&n) {
                    return Some(0x6F + n);
                }
            }
        }
        if s.len() == 1 {
            let c = s.to_uppercase().chars().next()?;
            if c.is_ascii_alphabetic() {
                return Some(c as u32);
            }
            if c.is_ascii_digit() {
                return Some(0x30 + (c as u32 - b'0' as u32));
            }
        }
        match s.to_lowercase().as_str() {
            "space" => Some(0x20),
            "enter" | "return" => Some(0x0D),
            "esc" | "escape" => Some(0x1B),
            "tab" => Some(0x09),
            "backspace" => Some(0x08),
            "delete" | "del" => Some(0x2E),
            "insert" | "ins" => Some(0x2D),
            "home" => Some(0x24),
            "end" => Some(0x23),
            "pageup" | "pgup" => Some(0x21),
            "pagedown" | "pgdn" => Some(0x22),
            "up" => Some(0x26),
            "down" => Some(0x28),
            "left" => Some(0x25),
            "right" => Some(0x27),
            _ => None,
        }
    }
}

// ── Win32 WH_KEYBOARD_LL hook ─────────────────────────────
#[cfg(target_os = "windows")]
mod win32 {
    use std::ffi::c_void;

    use super::*;

    type HOOKPROC = unsafe extern "system" fn(i32, usize, isize) -> isize;
    type HHOOK = isize;
    type HINSTANCE = *mut c_void;

    const WH_KEYBOARD_LL: i32 = 13;
    const HC_ACTION: i32 = 0;
    const WM_KEYDOWN: usize = 0x0100;
    const WM_SYSKEYDOWN: usize = 0x0104;

    #[repr(C)]
    struct KBDLLHOOKSTRUCT {
        vkCode: u32,
        scanCode: u32,
        flags: u32,
        time: u32,
        dwExtraInfo: usize,
    }

    extern "system" {
        fn SetWindowsHookExW(
            idHook: i32,
            lpfn: HOOKPROC,
            hmod: HINSTANCE,
            dwThreadId: u32,
        ) -> HHOOK;
        fn CallNextHookEx(
            hhk: HHOOK,
            nCode: i32,
            wParam: usize,
            lParam: isize,
        ) -> isize;
        fn UnhookWindowsHookEx(hhk: HHOOK) -> i32;
        fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
        fn GetAsyncKeyState(vKey: i32) -> i16;
        pub(crate) fn keybd_event(
            bVk: u8,
            bScan: u8,
            dwFlags: u32,
            dwExtraInfo: usize,
        );
    }

    pub(crate) fn unhook(hook: HHOOK) {
        unsafe { UnhookWindowsHookEx(hook); }
    }

    unsafe extern "system" fn hook_proc(
        nCode: i32,
        wParam: usize,
        lParam: isize,
    ) -> isize {
        if nCode == HC_ACTION
            && (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN)
        {
            let kb = &*(lParam as *const KBDLLHOOKSTRUCT);
            handle_keydown(kb.vkCode);
        }
        unsafe {
            let hhk = HOOK_HANDLE
                .get()
                .and_then(|m| m.lock().ok())
                .and_then(|g| *g)
                .unwrap_or(0);
            CallNextHookEx(hhk, nCode, wParam, lParam)
        }
    }

    fn handle_keydown(vk: u32) {
        let app = match APP.get() {
            Some(a) => a,
            None => return,
        };
        let data = match app.try_state::<AppData>() {
            Some(d) => d,
            None => return,
        };
        let settings = match data.settings.lock() {
            Ok(s) => s,
            Err(_) => return,
        };

        // Check show hotkey
        if let Some((target_vk, target_mods)) =
            parser::parse_hotkey(&settings.show_hotkey)
        {
            if vk == target_vk && check_mods(target_mods) {
                drop(settings);
                send_action(Action::ShowQuote);
                return;
            }
        }

        // Check collect hotkey
        if let Some((target_vk, target_mods)) =
            parser::parse_hotkey(&settings.collect_hotkey)
        {
            if vk == target_vk && check_mods(target_mods) {
                drop(settings);
                send_action(Action::Collect);
            }
        }
    }

    fn send_action(action: Action) {
        if let Some(mtx) = TX.get() {
            if let Ok(tx) = mtx.lock() {
                let _ = tx.send(action);
            }
        }
    }

    fn check_mods(desired: u32) -> bool {
        let ctrl = is_key_down(0x11);
        let alt = is_key_down(0x12);
        let shift = is_key_down(0x10);
        let win = is_key_down(0x5B) || is_key_down(0x5C);

        let actual = (if ctrl { 0x0002 } else { 0 })
            | (if alt { 0x0001 } else { 0 })
            | (if shift { 0x0004 } else { 0 })
            | (if win { 0x0008 } else { 0 });
        actual == desired
    }

    fn is_key_down(vk: i32) -> bool {
        unsafe { (GetAsyncKeyState(vk) as i16) < 0 }
    }

    pub fn install() {
        let hmod = unsafe { GetModuleHandleW(std::ptr::null()) };
        let hook = unsafe {
            SetWindowsHookExW(WH_KEYBOARD_LL, hook_proc, hmod, 0)
        };
        if hook != 0 {
            let _ = HOOK_HANDLE
                .get_or_init(|| std::sync::Mutex::new(None))
                .lock()
                .map(|mut h| *h = Some(hook));
        }
    }
}

fn install_hook_ll() {
    #[cfg(target_os = "windows")]
    win32::install();
}
