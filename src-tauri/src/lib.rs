mod hook;
mod remap;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{SetWindowsHookExW, WH_KEYBOARD_LL};

#[tauri::command]
fn close_keymap(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![close_keymap])
        .setup(|app| {
            unsafe {
                let hinstance: HMODULE = GetModuleHandleW(None)?;
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook::hook_proc), hinstance, 0)?;
            }

            let show_i = MenuItem::with_id(app, "keymap", "顯示 Keymap", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "結束", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("VimCaps - CapsLock 組合鍵")
                .menu(&menu)
                .show_menu_on_left_click(false) // 左鍵留給我們自己控制顯示/隱藏視窗，右鍵才跳選單
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "keymap" => show_keymap(app),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 單擊圖示：視窗有開就關、沒開就開
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                show_keymap(app);
                            }
                        }
                    }
                })
                .build(app)?;

            // 一啟動先把視窗藏起來，只留系統匣圖示
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        // 視窗被要求關閉（例如 Alt+F4）時只是隱藏，不真的結束程式，
        // 要結束要走系統匣選單的「結束」
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("啟動 tauri app 失敗");
}

fn show_keymap(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
