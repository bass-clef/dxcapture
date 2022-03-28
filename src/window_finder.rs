/// author: Robert Mikhayelyan <rob.mikh@outlook.com>

use winapi::{
    shared::{
        minwindef::{BOOL, DWORD, LPARAM},
        windef::HWND,
    },
    um::{
        dwmapi::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWM_CLOAKED_SHELL},
        wincon::{GetConsoleTitleW, SetConsoleTitleW},
        winuser::{
            EnumWindows, GetAncestor, GetClassNameW, GetShellWindow, GetWindowLongW,
            GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, GA_ROOT, GWL_EXSTYLE, GWL_STYLE,
            WS_DISABLED, WS_EX_TOOLWINDOW,
        },
    },
};

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub handle: HWND,
    pub title: String,
    pub class_name: String,
}

fn get_shell_window() -> HWND {
    unsafe { GetShellWindow() }
}

fn is_window_visible(window: HWND) -> bool {
    unsafe { IsWindowVisible(window) == 1 }
}

fn is_root_window(window: HWND) -> bool {
    unsafe { GetAncestor(window, GA_ROOT) == window }
}

fn match_title_and_class_name(window: &WindowInfo, title: &str, class_name: &str) -> bool {
    window.title == title && window.class_name == class_name
}

fn is_known_blocked_window(window: &WindowInfo) -> bool {
    match_title_and_class_name(window, "Task View", "Windows.UI.Core.CoreWindow")
        || match_title_and_class_name(
            window,
            "DesktopWindowXamlSource",
            "Windows.UI.Core.CoreWindow",
        )
        || match_title_and_class_name(window, "PopupHost", "Xaml_WindowedPopupClass")
}

fn is_capturable_window(window: &WindowInfo) -> bool {
    if window.title.is_empty()
        || window.handle == get_shell_window()
        || !is_window_visible(window.handle)
        || !is_root_window(window.handle)
    {
        return false;
    }

    let style = unsafe { GetWindowLongW(window.handle, GWL_STYLE) as u32 };
    if style & WS_DISABLED > 0 {
        return false;
    }

    let ex_style = unsafe { GetWindowLongW(window.handle, GWL_EXSTYLE) as u32 };
    if ex_style & WS_EX_TOOLWINDOW > 0 {
        return false;
    }

    if window.class_name == "Windows.UI.Core.CoreWindow"
        || window.class_name == "ApplicationFrameWindow"
    {
        let mut cloaked = 0;
        let result = unsafe {
            winrt::ErrorCode(DwmGetWindowAttribute(
                window.handle,
                DWMWA_CLOAKED,
                &mut cloaked as *mut _ as *mut _,
                std::mem::size_of::<DWORD>() as u32,
            ) as u32)
            .ok()
        };
        if let Ok(_) = result {
            if cloaked == DWM_CLOAKED_SHELL {
                return false;
            }
        }
    }

    if is_known_blocked_window(window) {
        return false;
    }

    return true;
}

extern "system" fn enum_window(handle: HWND, lparam: LPARAM) -> BOOL {
    let window_text_length = unsafe { GetWindowTextLengthW(handle) };
    if window_text_length > 0 {
        let window_text = unsafe {
            let window_text_length = window_text_length + 1;
            let mut text_array = vec![0u16; window_text_length as usize];
            GetWindowTextW(
                handle,
                text_array.as_mut_ptr() as *mut _,
                window_text_length,
            );
            std::string::String::from_utf16_lossy(&text_array)
                .trim_matches(char::from(0))
                .to_string()
        };
        let class_name = unsafe {
            let class_text_length: i32 = 256;
            let mut text_array = vec![0u16; class_text_length as usize];
            GetClassNameW(handle, text_array.as_mut_ptr() as *mut _, class_text_length);
            std::string::String::from_utf16_lossy(&text_array)
                .trim_matches(char::from(0))
                .to_string()
        };
        let info = WindowInfo {
            handle: handle,
            title: window_text,
            class_name: class_name,
        };

        if !is_capturable_window(&info) {
            return 1;
        }

        unsafe {
            let list = std::mem::transmute::<LPARAM, *mut Vec<WindowInfo>>(lparam);
            (*list).push(info);
        };
    }

    return 1;
}

pub fn get_capturable_windows() -> Vec<WindowInfo> {
    // https://support.microsoft.com/en-us/help/124103/how-to-obtain-a-console-window-handle-hwnd
    let current_console_title = unsafe {
        let console_title_length: u32 = 256;
        let mut text_array = vec![0u16; console_title_length as usize];
        GetConsoleTitleW(text_array.as_mut_ptr() as *mut _, console_title_length);
        std::string::String::from_utf16_lossy(&text_array)
            .trim_matches(char::from(0))
            .to_string()
    };

    unsafe {
        let temp_guid = uuid::Uuid::new_v4();
        let mut new_console_title: Vec<u16> = temp_guid.to_string().encode_utf16().collect();
        new_console_title.push(0);
        SetConsoleTitleW(new_console_title.as_mut_ptr() as *mut _);
    };
    let duration = std::time::Duration::from_millis(40);
    std::thread::sleep(duration);

    let mut window_list = Vec::<WindowInfo>::new();
    let result = unsafe { EnumWindows(Some(enum_window), &mut window_list as *mut _ as _) };
    if result == 0 {
        panic!("EnumWindows failed!");
        // TODO: GetLastError
        // TODO: ErrorCode conversion
    }

    unsafe {
        let mut new_console_title: Vec<u16> = current_console_title.encode_utf16().collect();
        new_console_title.push(0);
        SetConsoleTitleW(new_console_title.as_mut_ptr() as *mut _);
    };

    window_list
}

pub fn find_window(window_name: &str) -> Vec<WindowInfo> {
    let window_list = get_capturable_windows();
    let mut windows: Vec<WindowInfo> = Vec::new();
    for window_info in &window_list {
        let title = window_info.title.to_lowercase();
        if title.contains(&window_name.to_string().to_lowercase()) {
            windows.push(window_info.clone());
        }
    }
    windows
}
