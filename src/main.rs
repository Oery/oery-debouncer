#![windows_subsystem = "windows"]
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
    static ref BUTTON_DATA: Mutex<HashMap<u32, ButtonState>> = Mutex::new(HashMap::new());
}

struct ButtonState {
    is_down: bool,
    last_press_time: u32,
}

static mut DEBOUNCE_TIME: u32 = 70;
unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let mouse_struct = *(lparam.0 as *const MSLLHOOKSTRUCT);
        let current_time = mouse_struct.time;

        let mut button_data = BUTTON_DATA.lock().unwrap();

        match wparam.0 as u32 {
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
                let button = match wparam.0 as u32 {
                    WM_LBUTTONDOWN => 0,
                    WM_RBUTTONDOWN => 1,
                    WM_MBUTTONDOWN => 2,
                    _ => return CallNextHookEx(None, code, wparam, lparam),
                };

                if let Some(button_state) = button_data.get_mut(&button) {
                    if !button_state.is_down {
                        if current_time - button_state.last_press_time < DEBOUNCE_TIME {
                            return LRESULT(1); // Filter out this click
                        }
                        button_state.is_down = true;
                        button_state.last_press_time = current_time; // Update time only for non-filtered clicks
                    }
                } else {
                    // First time this button is pressed
                    button_data.insert(
                        button,
                        ButtonState {
                            is_down: true,
                            last_press_time: current_time,
                        },
                    );
                }
            }
            WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => {
                let button = match wparam.0 as u32 {
                    WM_LBUTTONUP => 0,
                    WM_RBUTTONUP => 1,
                    WM_MBUTTONUP => 2,
                    _ => return CallNextHookEx(None, code, wparam, lparam),
                };

                if let Some(button_state) = button_data.get_mut(&button) {
                    button_state.is_down = false;
                    // We don't update last_press_time here anymore
                }
            }
            _ => {}
        };
    }
    CallNextHookEx(None, code, wparam, lparam)
}

fn set_mouse_hook() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        let hook = SetWindowsHookExA(WH_MOUSE_LL, Some(mouse_hook_proc), instance, 0);

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
    let mut tray = TrayItem::new("Oery Mouse Debouncer", IconSource::Resource("icon")).unwrap();
    tray.add_label("Oery Mouse Debouncer").unwrap();

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
        set_mouse_hook().unwrap();
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
