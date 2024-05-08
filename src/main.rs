use std::{
    collections::HashMap,
    sync::{mpsc, Mutex},
    thread,
};

use tray_item::{IconSource, TrayItem};
use windows::{
    core::*,
    Win32::{Foundation::*, System::LibraryLoader::GetModuleHandleA, UI::WindowsAndMessaging::*},
};

use lazy_static::lazy_static;

lazy_static! {
    static ref LAST_KEY_PRESSES: Mutex<HashMap<u32, u32>> = Mutex::new(HashMap::new());
}

const DEBOUNCE_TIME: u32 = 70;

unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let vk_code = kb_struct.vkCode;

        if wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize {
            let current_time = kb_struct.time;
            let mut map = LAST_KEY_PRESSES.lock().unwrap();
            if let Some(&last_press_time) = map.get(&vk_code) {
                if current_time - last_press_time < DEBOUNCE_TIME {
                    return LRESULT(1);
                }
            }
            map.insert(vk_code, current_time);
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

fn set_keyboard_hook() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        let hook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_hook_proc), instance, 0);

        let mut msg = MSG::default();

        while GetMessageA(&mut msg, HWND(0), 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }

        let _ = UnhookWindowsHookEx(hook.unwrap());
    }
    Ok(())
}

enum Message {
    Quit,
}

fn main() {
    let mut tray = TrayItem::new("Oery Debouncer", IconSource::Resource("icon")).unwrap();
    tray.add_label("Oery Debouncer").unwrap();

    let (tx, rx) = mpsc::sync_channel(1);

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    thread::spawn(move || {
        set_keyboard_hook().unwrap();
    });

    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                break;
            }
            Err(_) => todo!(),
        }
    }
}
