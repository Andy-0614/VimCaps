mod hook;
mod remap;

use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, SetWindowsHookExW, TranslateMessage, WH_KEYBOARD_LL,
};

fn main() {
    unsafe {
        let hinstance: HMODULE = GetModuleHandleW(None).expect("GetModuleHandleW failed");

        SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook::hook_proc), hinstance, 0)
            .expect("SetWindowsHookExW failed");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
