use std::mem::{zeroed, transmute};
use std::ptr::{null_mut};
use std::sync::atomic::{AtomicPtr, Ordering};
use winapi::shared::{minwindef::{HINSTANCE, LPARAM, DWORD}, windef::HWND};
use winapi::ctypes::{c_void, c_int, c_uint};
use winapi::um::winuser::{WM_USER, SendMessageW, SetTimer};
use winapi::um::consoleapi::AllocConsole;
// use widestring::U16CString;

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

    let result = unsafe { transmute::<isize, *mut c_void>(SendMessageW(parent, WM_WA_IPC, 0, IPC_GETVUDATAFUNC)) };
    VUDATAFUNC.store(result, Ordering::Relaxed);
    println!("Result: {:?}", result);

    unsafe { SetTimer(null_mut(), 0, 33, Some(on_timer)) };

    0x0
}

extern fn config() {}

extern fn quit() {}

extern "system" fn on_timer(_: HWND, _: c_uint, _: usize, _: DWORD) {
    let vu1 = vu_get(0);
    let vu2 = vu_get(1);

    println!("Channel 0: {:5} Channel 1: {:5}", vu1, vu2);
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
