use winapi::{
    um::{
        d3d11::{
            D3D11CreateDevice,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            D3D11_SDK_VERSION,
        },
        d3dcommon::{
            D3D_DRIVER_TYPE_HARDWARE,
        },
    },
    winrt::roapi::{
        RoInitialize,
        RO_INIT_MULTITHREADED,
    },
};
use windows::{
    core::Interface,
    Graphics::{
        Capture::GraphicsCaptureItem,
        DirectX::Direct3D11::{
            IDirect3DDevice,
            IDirect3DSurface,
        },
    },
    Win32::{
        Foundation::HWND,
        Graphics::{
            Direct3D11::{
                ID3D11Device,
                ID3D11DeviceContext,
                ID3D11Texture2D,
            },
            Dxgi::{
                IDXGIDevice,
                IDXGISurface,
            },
            Gdi::{
                MonitorFromWindow,
                HMONITOR,
                MONITOR_DEFAULTTOPRIMARY,
            },
        },
        UI::WindowsAndMessaging::{
            GetDesktopWindow,
        },
        System::WinRT::{
            Direct3D11::{
                CreateDirect3D11DeviceFromDXGIDevice,
                CreateDirect3D11SurfaceFromDXGISurface,
                IDirect3DDxgiInterfaceAccess,
            },
            Graphics::Capture::{
                IGraphicsCaptureItemInterop,
            },
        },
    }
};
use winrt::AbiTransferable;

pub struct D3D11Device;
impl D3D11Device {
    fn new_of_type() -> winrt::Result<ID3D11Device> {
        let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;

        Ok(unsafe {
            let mut device = winrt::IUnknown::default();
            winrt::ErrorCode(D3D11CreateDevice(
                std::ptr::null_mut(),
                D3D_DRIVER_TYPE_HARDWARE,
                std::ptr::null_mut(),
                flags,
                std::ptr::null(),
                0,
                D3D11_SDK_VERSION,
                device.set_abi() as *mut *mut _,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) as u32).ok()?;

            std::mem::transmute(device)
        })
    }

    fn to_direct3d_device(device: &ID3D11Device) -> windows::core::Result<IDirect3DDevice> {
        unsafe {
            let dxgi_device: IDXGIDevice = device.cast::<IDXGIDevice>()?;
            let result: IDirect3DDevice = CreateDirect3D11DeviceFromDXGIDevice(dxgi_device)?.cast::<IDirect3DDevice>()?;

            Ok(result)
        }
    }
}


#[derive(Debug)]
pub struct Device {
    pub d3d_device: ID3D11Device,
    pub device: IDirect3DDevice,
    pub item: GraphicsCaptureItem,
}
impl Device {
    /// Create a new Device.
    /// 
    /// other all in common initialize. other is self made case for [GraphicsCaptureItem].
    pub fn new(item: GraphicsCaptureItem) -> Self {
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED);
        }

        let d3d_device = D3D11Device::new_of_type().unwrap();
        let device = D3D11Device::to_direct3d_device(&d3d_device).unwrap();

        Self {
            d3d_device,
            device,
            item,
        }
    }

    /// Create Device from display id.
    /// ## Parameters
    /// * display_id: id of the target display. default is created by [MONITOR_DEFAULTTOPRIMARY](winapi::um::winuser::MONITOR_DEFAULTTOPRIMARY).
    /// display_id range is [1..=len].
    pub fn new_from_displays(display_id: Option<usize>) -> anyhow::Result<Self> {
        let monitor_handle = if let Some(display_id) = display_id {
            let displays = crate::displays::enumerate_displays();
            if display_id <= 0 || displays.len() <= display_id - 1 {
                return Err(anyhow::anyhow!("DisplayId is out of range"));
            }

            HMONITOR{ 0: displays[display_id].handle as isize }
        } else {
            unsafe{ MonitorFromWindow(GetDesktopWindow(), MONITOR_DEFAULTTOPRIMARY) }
        };

        let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
        let item: GraphicsCaptureItem = unsafe{ interop.CreateForMonitor(monitor_handle)? };
        Ok(Self::new( item ))
    }

    /// Create Device from window caption.
    /// ## Parameters
    /// * window_caption: Window caption of the target window. default is created by [GetDesktopWindow].
    pub fn new_from_window(window_caption: String) -> anyhow::Result<Self> {
        let window_handle = {
            let windows = crate::window_finder::find_window(&window_caption);
            if windows.len() == 0 {
                anyhow::bail!("Window is not found");
            }

            HWND { 0: windows[0].handle as isize }
        };

        let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
        let item: GraphicsCaptureItem = unsafe{ interop.CreateForWindow(window_handle)? };
        Ok(Self::new( item ))
    }

    pub fn get_immediate_context(d3d_device: &ID3D11Device) -> windows::core::Result<ID3D11DeviceContext> {
        Ok(unsafe {
            let mut d3d_context: Option<ID3D11DeviceContext> = Some(
                std::mem::transmute::<_, ID3D11DeviceContext>(winrt::IUnknown::default())
            );
            d3d_device.GetImmediateContext( &mut d3d_context );
    
            d3d_context.unwrap()
        })
    }

    pub fn to_direct3d_surface(texture: &ID3D11Texture2D) -> windows::core::Result<IDirect3DSurface> {
        let dxgi_surface = texture.cast::<IDXGISurface>()?;

        unsafe {
            CreateDirect3D11SurfaceFromDXGISurface(dxgi_surface)?.cast::<IDirect3DSurface>()
        }
    }

    pub fn from_direct3d_surface(surface: &IDirect3DSurface) -> windows::core::Result<ID3D11Texture2D> {
        let access = surface.cast::<IDirect3DDxgiInterfaceAccess>()?;

        unsafe {
            access.GetInterface::<ID3D11Texture2D>()
        }
    }
}

impl Default for Device {
    /// Create a new Device with primary monitor.
    fn default() -> Self {
        Self::new_from_displays(None).expect("Not found primary monitor")
    }
}