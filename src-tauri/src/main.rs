// Windows release build 時不要跳出黑色主控台視窗
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    vimcaps_lib::run();
}
