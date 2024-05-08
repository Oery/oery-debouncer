mod config;

use std::{
    collections::HashMap,
    sync::{mpsc, Mutex},
    thread,
};

use config::{load_config, save_config, Config};
use tray_item::{IconSource, TrayItem};
use windows::{
    core::*,
    Win32::{Foundation::*, System::LibraryLoader::GetModuleHandleA, UI::WindowsAndMessaging::*},
};

use lazy_static::lazy_static;

lazy_static! {
    static ref LAST_KEY_PRESSES: Mutex<HashMap<u32, u32>> = Mutex::new(HashMap::new());
}

static mut DEBOUNCE_TIME: u32 = 70;

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

fn loword(value: u32) -> u16 {
    (value & 0xffff) as u16
}

unsafe extern "system" fn dialog_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> isize {
    match msg {
        WM_INITDIALOG => {
            SetDlgItemInt(hwnd, 1001, DEBOUNCE_TIME, BOOL(0)).unwrap();
            TRUE.0 as isize
        }
        WM_COMMAND => {
            if loword(wparam.0 as u32) == IDOK.0 as u16 {
                // Update DEBOUNCE_TIME from text box when OK is clicked
                let mut success = BOOL(0);
                let new_time = GetDlgItemInt(hwnd, 1001, Some(&mut success), BOOL(0));
                if success.as_bool() {
                    DEBOUNCE_TIME = new_time;
                    let _ = save_config(&Config {
                        debounce_time: new_time,
                    });
                }
                EndDialog(hwnd, 0).unwrap();
            }
            0
        }
        _ => 0,
    }
}

enum Message {
    Quit,
}

fn main() {
    let mut tray = TrayItem::new("Oery Debouncer", IconSource::Resource("icon")).unwrap();
    tray.add_label("Oery Debouncer").unwrap();

    let (tx, rx) = mpsc::sync_channel(1);

    tray.add_menu_item("Change Debounce Time", move || unsafe {
        let instance = GetModuleHandleA(None).unwrap();
        let template_name = PSTR(b"DEBOUNCE_TIME_DIALOG\0".as_ptr() as _);
        DialogBoxParamA(instance, template_name, None, Some(dialog_proc), LPARAM(0));
    })
    .unwrap();

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    let config = match load_config() {
        Ok(config) => config,
        Err(_) => Config { debounce_time: 70 },
    };

    unsafe {
        DEBOUNCE_TIME = config.debounce_time;
    }

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
