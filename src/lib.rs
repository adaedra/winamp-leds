use std::mem::{zeroed, transmute};
use std::ptr::{null_mut};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::cmp::max;
use winapi::shared::{minwindef::{HINSTANCE, LPARAM, DWORD}, windef::HWND};
use winapi::ctypes::{c_void, c_int, c_uint};
use winapi::um::winuser::{WM_USER, SendMessageW, SetTimer};
use winapi::um::consoleapi::AllocConsole;
// use widestring::U16CString;

mod corsair;
use corsair::Color;

#[repr(C)]
pub struct WinampGeneralPurposePlugin {
    version: c_int,
    description: *const u8,
    init: extern fn() -> c_int,
    config: extern fn() -> (),
    quit: extern fn() -> (),
    parent: HWND,
    dll_instance: HINSTANCE,
}

static PLUGIN: AtomicPtr<WinampGeneralPurposePlugin> = AtomicPtr::new(null_mut());
static VUDATAFUNC: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

fn get_plugin<'a>() -> &'a WinampGeneralPurposePlugin {
    unsafe { &*PLUGIN.load(Ordering::Relaxed) }
}

fn vu_get(channel: i32) -> i32 {
    let wa_vu_get: unsafe fn(c_int) -> c_int = unsafe { transmute(VUDATAFUNC.load(Ordering::Relaxed)) };

    unsafe { wa_vu_get(channel as c_int) as i32 }
}

const WM_WA_IPC: c_uint = WM_USER;
const IPC_GETVUDATAFUNC: LPARAM = 801;

extern fn init() -> c_int {
    let parent = get_plugin().parent;
    unsafe { AllocConsole() };

    if !corsair::handshake() { return 0x1; }

    let devices = corsair::devices();
    println!("{:?}", devices);

    let result = unsafe { transmute::<isize, *mut c_void>(SendMessageW(parent, WM_WA_IPC, 0, IPC_GETVUDATAFUNC)) };
    VUDATAFUNC.store(result, Ordering::Relaxed);
    println!("Result: {:?}", result);

    unsafe { SetTimer(null_mut(), 0, 33, Some(on_timer)) };

    0x0
}

extern fn config() {}

extern fn quit() {}

extern "system" fn on_timer(_: HWND, _: c_uint, _: usize, _: DWORD) {
    let vu_left = vu_get(0);
    let vu_right = vu_get(1);
    let value = max(vu_left, vu_right);

    if value == -1 {
        return;
    }

    let left_width = vu_left * 15 / 255;
    let right_width = vu_right * 15 / 255;

    let mut front_leds = Vec::with_capacity(30);
    for idx in 0 .. 15 - left_width {
        front_leds.push((200 + idx, Color::new(255, 255, 255)));
    }
    for idx in 0 .. left_width + right_width {
        front_leds.push((200 + 15 - left_width + idx, Color::new(255, 0, 0)));
    }
    for idx in 0 .. 15 - right_width {
        front_leds.push((230 - idx - 1, Color::new(255, 255, 255)));
    }

    // println!("{:?}", front_leds);
    if front_leds.len() != 30 {
        println!("ERR: {} items ({}, {})", front_leds.len(), left_width, right_width);
    } else {
        corsair::set_leds(0, &front_leds[..]);
    }

    let mut cpu_leds = Vec::with_capacity(12);
    for idx in 0 .. 12 {
        cpu_leds.push((766 + idx, Color::new(value as u8, value as u8, value as u8)));
    }
    corsair::set_leds(1, &cpu_leds[..]);

    corsair::flush();
}

#[no_mangle]
pub extern fn winampGetGeneralPurposePlugin() -> *mut WinampGeneralPurposePlugin {
    let plugin = Box::new(WinampGeneralPurposePlugin {
        version: 0x10,
        // Ugly!
        description: b"LED Control\0" as *const u8,
        init: init,
        config: config,
        quit: quit,
        parent: unsafe { zeroed() },
        dll_instance: unsafe { zeroed() }
    });

    let addr = Box::leak(plugin);
    PLUGIN.store(addr, Ordering::Relaxed);
    addr as *mut WinampGeneralPurposePlugin
}
