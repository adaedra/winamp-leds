use std::mem::{zeroed, transmute};
use std::ptr::{null_mut};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ffi::CStr;
use std::cmp::max;
use winapi::shared::{minwindef::{HINSTANCE, LPARAM, DWORD}, windef::HWND};
use winapi::ctypes::{c_void, c_char, c_int, c_uint};
use winapi::um::winuser::{WM_USER, SendMessageW, SetTimer};
use winapi::um::consoleapi::AllocConsole;
// use widestring::U16CString;

#[repr(C)]
pub struct CorsairChannelsInfo {
    channels_count: c_int,
    channels: *const c_void,
}

#[repr(C)]
pub struct CorsairProtocolDetails {
    sdk_version: *const c_char,
    server_version: *const c_char,
    sdk_protocol_version: c_int,
    server_protocl_version: c_int,
    breaking_changes: bool,
    channels: CorsairChannelsInfo,
}

#[repr(C)]
#[derive(Debug)]
pub struct CorsairDeviceInfo {
    device_type: c_int,
    model: *const c_char,
    physical_layout: c_int,
    logical_layout: c_int,
    caps_mask: c_int,
    leds_count: c_int,
}

#[repr(C)]
pub struct CorsairLedColor {
    led_id: c_int,
    red: c_int,
    green: c_int,
    blue: c_int,
}

#[link(name = "CUESDK_2017")]
extern "C" {
    fn CorsairPerformProtocolHandshake() -> CorsairProtocolDetails;
    fn CorsairGetDeviceCount() -> c_int;
    fn CorsairGetDeviceInfo(device_index: c_int) -> *const CorsairDeviceInfo;
    fn CorsairSetLedsColorsBufferByDeviceIndex(device_index: c_int, size: c_int, leds_colors: *const CorsairLedColor);
    fn CorsairSetLedsColorsFlushBuffer() -> bool;
}

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

    let corsair = unsafe { CorsairPerformProtocolHandshake() };
    println!("SDK {}, Server {}", corsair.sdk_protocol_version, corsair.server_protocl_version);

    if corsair.server_protocl_version == 0 {
        return 0x1;
    }

    let device_count = unsafe { CorsairGetDeviceCount() };
    println!("{} devices", device_count);

    for idx in 0 .. device_count {
        let device = unsafe { &*CorsairGetDeviceInfo(idx) };
        let name = unsafe { CStr::from_ptr(device.model).to_str().unwrap() };

        println!("{}: {:?}", name, device);
    }

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

    let color = CorsairLedColor {
        led_id: 762,
        red: max(0, value) / 2,
        green: max(0, value) / 2,
        blue: max(0, value)
    };

    unsafe {
        CorsairSetLedsColorsBufferByDeviceIndex(0, 1, &color as *const CorsairLedColor);
        CorsairSetLedsColorsFlushBuffer();
    }
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
