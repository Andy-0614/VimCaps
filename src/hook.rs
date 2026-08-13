use std::collections::BTreeSet;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, KBDLLHOOKSTRUCT, LLKHF_INJECTED, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP,
};

use crate::remap;

static HELD: Mutex<BTreeSet<u32>> = Mutex::new(BTreeSet::new());
static ACTIVE_COMBO: Mutex<Option<(&'static [u32], &'static [u32])>> = Mutex::new(None);

// 三段式序列組合用的狀態：CURRENT_STAGE 是這一段還沒放開的鍵，SEQUENCE 是已完成的段落
static CURRENT_STAGE: Mutex<BTreeSet<u32>> = Mutex::new(BTreeSet::new());
static SEQUENCE: Mutex<Vec<Vec<u32>>> = Mutex::new(Vec::new());
static LAST_STAGE_AT: Mutex<Option<Instant>> = Mutex::new(None);

const MAX_SIMULTANEOUS_KEYS: usize = 4;
const VK_CAPITAL: u32 = 0x14;
const SEQUENCE_TIMEOUT: Duration = Duration::from_millis(1500);

pub unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };

        // 忽略我們自己用 SendInput 送出的合成按鍵，避免遞迴鎖死、也避免合成鍵被自己的攔截邏輯誤擋
        if kb.flags & LLKHF_INJECTED == LLKHF_INJECTED {
            return unsafe { CallNextHookEx(None, code, wparam, lparam) };
        }

        let mut held = HELD.lock().unwrap();

        match wparam.0 as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                // CapsLock 攔截 + CapsLock 組合鍵攔截：CapsLock 本身一律攔截，且只要 CapsLock 正被按住，其他任何按鍵也一併攔截
                let capslock_held = held.contains(&VK_CAPITAL);
                let intercept = kb.vkCode == VK_CAPITAL || capslock_held;

                // 限制同時按鍵最多 4 個，並打印目前按下的 keycode 組合
                if held.len() < MAX_SIMULTANEOUS_KEYS && held.insert(kb.vkCode) {
                    CURRENT_STAGE.lock().unwrap().insert(kb.vkCode);

                    let line = held
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(" + ");
                    println!("{line}");

                    // 這一鍵剛好讓整組符合表中的組合，才觸發一次性依序執行多段 macro（跟按住多久無關）
                    if let Some(stages) = remap::lookup_macro(&held) {
                        remap::send_macro(stages);
                    }
                }

                // 目前按住的整組鍵剛好對上表裡的組合才送出對應組合，避免同一組合持續觸發
                let mut active_combo = ACTIVE_COMBO.lock().unwrap();
                if active_combo.is_none() {
                    if let Some((trigger, output)) = remap::lookup(&held) {
                        remap::send_combo(output, false);
                        *active_combo = Some((trigger, output));
                    }
                }

                // 按鍵無效化
                if intercept {
                    return LRESULT(1);
                }
            }
            WM_KEYUP | WM_SYSKEYUP => {
                held.remove(&kb.vkCode);

                // 組合裡任一鍵放開就代表組合中斷，把送出的組合也一併放開
                let mut active_combo = ACTIVE_COMBO.lock().unwrap();
                if let Some((trigger, output)) = *active_combo {
                    if trigger.contains(&kb.vkCode) {
                        remap::send_combo(output, true);
                        *active_combo = None;
                    }
                }

                // 這輪按下的鍵全部放開，代表一段結束，收進序列比對三段式組合（4 + 4 + 4）
                if held.is_empty() {
                    let mut current_stage = CURRENT_STAGE.lock().unwrap();
                    let stage: Vec<u32> = current_stage.iter().copied().collect();
                    current_stage.clear();

                    if !stage.is_empty() {
                        let mut sequence = SEQUENCE.lock().unwrap();
                        let mut last_stage_at = LAST_STAGE_AT.lock().unwrap();

                        // 段與段之間超過逾時就當作前面的序列作廢，從這段重新開始
                        if last_stage_at.is_some_and(|at| at.elapsed() > SEQUENCE_TIMEOUT) {
                            sequence.clear();
                        }

                        sequence.push(stage);
                        *last_stage_at = Some(Instant::now());

                        if let Some(output) = remap::lookup_sequence(&sequence) {
                            remap::send_combo(output, false);
                            remap::send_combo(output, true);
                            sequence.clear();
                        } else if !remap::is_sequence_prefix(&sequence) {
                            sequence.clear();
                        }
                    }
                }

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
