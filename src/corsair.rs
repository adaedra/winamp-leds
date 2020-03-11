use palette::rgb::Rgb;

use winapi::shared::{
    ntdef::{CHAR, VOID},
    minwindef::{BOOL, INT}
};

#[repr(C)]
pub struct CorsairChannelsInfo {
    channels_count: INT,
    channels: *const VOID, // TODO: Complete
}

#[repr(C)]
pub struct CorsairProtocolDetails {
    sdk_version: *const CHAR,
    server_version: *const CHAR,
    sdk_protocol_version: INT,
    server_protocl_version: INT,
    breaking_changes: BOOL,
    channels: CorsairChannelsInfo,
}

#[repr(C)]
pub struct CorsairDeviceInfo {
    device_type: INT,
    model: *const CHAR,
    physical_layout: INT,
    logical_layout: INT,
    caps_mask: INT,
    leds_count: INT,
}

#[repr(C)]
pub struct CorsairLedColor {
    led_id: INT,
    red: INT,
    green: INT,
    blue: INT,
}

#[derive(Debug)]
pub struct Device {
    id: usize,
    name: String,
    leds_count: usize,
}

#[link(name = "CUESDK_2017")]
extern "C" {
    fn CorsairPerformProtocolHandshake() -> CorsairProtocolDetails;
    fn CorsairGetDeviceCount() -> INT;
    fn CorsairGetDeviceInfo(device_index: INT) -> *const CorsairDeviceInfo;
    fn CorsairSetLedsColorsBufferByDeviceIndex(device_index: INT, size: INT, leds_colors: *const CorsairLedColor);
    fn CorsairSetLedsColorsFlushBuffer() -> BOOL;
}

pub fn handshake() -> bool {
    let data = unsafe { CorsairPerformProtocolHandshake() };

    data.server_protocl_version != 0
}

// TODO
pub fn devices() -> Vec<Device> {
    use std::ffi::CStr;

    let count = unsafe { CorsairGetDeviceCount() };
    let mut result = Vec::with_capacity(count as usize);

    for idx in 0 .. count {
        let device = unsafe { &*CorsairGetDeviceInfo(idx as INT) };

        result.push(Device {
            id: idx as usize,
            name: unsafe { CStr::from_ptr(device.model) }.to_str().unwrap().to_owned(),
            leds_count: device.leds_count as usize,
        });
    }

    result
}

// TODO: Check for errors
pub fn set_leds(device: i32, leds: &[(i32, Rgb)]) {
    let mut data = Vec::with_capacity(leds.len());

    for led in leds {
        let (id, Rgb { red, green, blue, .. }) = *led;

        data.push(CorsairLedColor {
            led_id: id as INT,
            red: (red * 255.0) as INT,
            green: (green * 255.0) as INT,
            blue: (blue * 255.0) as INT,
        });
    }

    unsafe {
        CorsairSetLedsColorsBufferByDeviceIndex(device, data.len() as INT, data.as_ptr());
    }
}

// TODO: Check for errors
pub fn flush() {
    unsafe {
        CorsairSetLedsColorsFlushBuffer();
    }
}
