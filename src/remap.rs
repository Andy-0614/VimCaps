use std::collections::BTreeSet;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput,
    VIRTUAL_KEY,
};

// 觸發組合 (左欄，最多 4 鍵) -> 要「持續按住」的組合 (右欄，最多 4 鍵)，觸發鍵放開才跟著放開，自行在這裡新增/修改
pub const REMAP_TABLE: &[(&[u32], &[u32])] = &[];

pub fn lookup(held: &BTreeSet<u32>) -> Option<(&'static [u32], &'static [u32])> {
    REMAP_TABLE
        .iter()
        .find(|(trigger, _)| trigger.len() == held.len() && trigger.iter().all(|k| held.contains(k)))
        .map(|(trigger, output)| (*trigger, *output))
}

// 觸發組合 (左欄，最多 4 鍵) -> 依序執行的最多 4 段輸出 (右欄)，每段本身也是最多 4 鍵的組合
// 每段都是「這段的鍵全部按下 -> 這段的鍵全部放開」才換下一段，觸發當下一次性送完，不受觸發鍵放開時間影響
pub const MACRO_TABLE: &[(&[u32], &[&[u32]])] = &[
    (&[0x14], &[&[0x1B]]),                     // CapsLock -> Esc (Normal)
    (&[0x14, 0x49], &[&[0x1B], &[0x49]]),      // CapsLock + i -> Esc, i (Edit)
    (&[0x14, 0x56], &[&[0x1B], &[0x56]]),      // CapsLock + v -> Esc, v (vis)
    (&[0x14, 0x52], &[&[0x1B], &[0x55]]),      // CapsLock + r -> Esc, u (undo)
    (&[0x14, 0x55], &[&[0x1B], &[0xA2, 0x52]]), // CapsLock + u -> Esc, ctrl + r (redo)
    (&[0x14, 0xBA], &[&[0x1B], &[0xA0, 0xBA]]), // CapsLock + ; -> Esc, : (command)
    (&[0x14, 0x46], &[&[0x1B], &[0xBF]]),      // CapsLock + f -> Esc, / (found)
    (
        &[0x14, 0x53],
        &[&[0x1B], &[0xA0, 0xBA], &[0x57], &[0x51], &[0xA0, 0x31]], // Esc, :, w, q, !
    ), // CapsLock + s -> Esc, :, w, q, !
    (
        &[0x14, 0x48],
        &[&[0x1B], &[0xA0, 0xBA], &[0xA0, 0x35], &[0x53], &[0xBF]], // Esc, :, %, s, /
    ), // CapsLock + h -> Esc, :, %, s, /
];

pub fn lookup_macro(held: &BTreeSet<u32>) -> Option<&'static [&'static [u32]]> {
    MACRO_TABLE
        .iter()
        .find(|(trigger, _)| trigger.len() == held.len() && trigger.iter().all(|k| held.contains(k)))
        .map(|(_, output)| *output)
}

pub fn send_macro(stages: &[&[u32]]) {
    for stage in stages {
        send_combo(stage, false);
        send_combo(stage, true);
    }
}

// 三段依序按下放開的組合 (左欄，每段最多 4 鍵) -> 要送出的組合 (右欄)，自行在這裡新增/修改
// 例如 (&[&[0x14, 0x48], &[0x14, 0x4A], &[0x14, 0x4B]], &[0x1B]) 表示
// 「CapsLock+H 放開」再「CapsLock+J 放開」再「CapsLock+K 放開」三段都按完才送出 Esc
pub const SEQUENCE_TABLE: &[(&[&[u32]], &[u32])] = &[];

pub fn lookup_sequence(sequence: &[Vec<u32>]) -> Option<&'static [u32]> {
    SEQUENCE_TABLE
        .iter()
        .find(|(stages, _)| stages_match(stages, sequence, true))
        .map(|(_, output)| *output)
}

pub fn is_sequence_prefix(sequence: &[Vec<u32>]) -> bool {
    SEQUENCE_TABLE
        .iter()
        .any(|(stages, _)| stages_match(stages, sequence, false))
}

fn stages_match(stages: &[&[u32]], sequence: &[Vec<u32>], exact_len: bool) -> bool {
    if stages.len() < sequence.len() || (exact_len && stages.len() != sequence.len()) {
        return false;
    }

    stages
        .iter()
        .zip(sequence.iter())
        .all(|(stage, done)| stage.len() == done.len() && stage.iter().all(|k| done.contains(k)))
}

pub fn send_combo(keys: &[u32], key_up: bool) {
    if key_up {
        for &vk in keys.iter().rev() {
            send_key(vk, true);
        }
    } else {
        for &vk in keys {
            send_key(vk, false);
        }
    }
}

fn send_key(vk: u32, key_up: bool) {
    let flags = if key_up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk as u16),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}
