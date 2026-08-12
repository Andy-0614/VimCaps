use std::collections::BTreeSet;
use std::sync::Mutex;

use windows::Win32::Foundation::{HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, MSG, SetWindowsHookExW,
    TranslateMessage, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static HELD: Mutex<BTreeSet<u32>> = Mutex::new(BTreeSet::new());

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };

        let mut held = HELD.lock().unwrap();

        match wparam.0 as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if held.insert(kb.vkCode) {
                    let line = held
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(" + ");
                    println!("{line}");
                }
            }
            WM_KEYUP | WM_SYSKEYUP => {
                held.remove(&kb.vkCode);
            }
            _ => {}
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn main() {
    unsafe {
        let hinstance: HMODULE = GetModuleHandleW(None).expect("GetModuleHandleW failed");

        SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hinstance, 0)
            .expect("SetWindowsHookExW failed");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
