/// author: Robert Mikhayelyan <rob.mikh@outlook.com>

use winapi::{
    shared::{
        minwindef::{BOOL, LPARAM},
        windef::{HDC, HMONITOR, LPRECT},
    },
    um::winuser::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW},
};

#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub handle: HMONITOR,
    pub display_name: String,
}

extern "system" fn enum_monitor(handle: HMONITOR, _: HDC, _: LPRECT, lparam: LPARAM) -> BOOL {
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    let result = unsafe { GetMonitorInfoW(handle, &mut monitor_info as *mut _ as *mut _) };
    if result == 0 {
        panic!("GetMonitorInfoW failed!");
        // TODO: GetLastError
        // TODO: ErrorCode conversion
    }

    let display_name = String::from_utf16_lossy(&monitor_info.szDevice)
        .trim_matches(char::from(0))
        .to_string();

    let info = DisplayInfo {
        handle: handle,
        display_name: display_name,
    };

    unsafe {
        let list = std::mem::transmute::<LPARAM, *mut Vec<DisplayInfo>>(lparam);
        (*list).push(info);
    };

    return 1;
}

/// Get all displays and returns them as a Vec.
pub fn enumerate_displays() -> Vec<DisplayInfo> {
    let mut displays: Vec<DisplayInfo> = Vec::new();
    let result = unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            Some(enum_monitor),
            &mut displays as *mut _ as _,
        )
    };
    if result == 0 {
        panic!("EnumDisplayMonitors failed!");
        // TODO: GetLastError
        // TODO: ErrorCode conversion
    }
    displays
}
