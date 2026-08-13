use std::collections::BTreeSet;
use std::sync::Mutex;

use windows::Win32::Foundation::{HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, MSG, SetWindowsHookExW,
    TranslateMessage, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static HELD: Mutex<BTreeSet<u32>> = Mutex::new(BTreeSet::new());

const MAX_SIMULTANEOUS_KEYS: usize = 4;
const VK_CAPITAL: u32 = 0x14;

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };

        let mut held = HELD.lock().unwrap();

        match wparam.0 as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                // CapsLock 攔截 + CapsLock 組合鍵攔截：CapsLock 本身一律攔截，且只要 CapsLock 正被按住，其他任何按鍵也一併攔截
                let intercept = kb.vkCode == VK_CAPITAL || held.contains(&VK_CAPITAL);

                // 限制同時按鍵最多 4 個，並打印目前按下的 keycode 組合
                if held.len() < MAX_SIMULTANEOUS_KEYS && held.insert(kb.vkCode) {
                    let line = held
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(" + ");
                    println!("{line}");
                }

                if intercept {
                    return LRESULT(1);
                }
            }
            WM_KEYUP | WM_SYSKEYUP => {
                held.remove(&kb.vkCode);

                // CapsLock 無效化：放開時也攔截，避免觸發原本的大小寫燈號切換
                if kb.vkCode == VK_CAPITAL {
                    return LRESULT(1);
                }
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