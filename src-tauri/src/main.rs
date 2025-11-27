// Binary entry point for desktop-proofer
// This simply delegates to the library's run() function

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    desktop_proofer_lib::run();
}
