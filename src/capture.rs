use std::sync::{
    Arc,
    Mutex
};
use winapi::{
    shared::dxgiformat::{
        DXGI_FORMAT_B8G8R8A8_UNORM,
    },
    um::d3d11::{
        D3D11_CPU_ACCESS_READ,
        D3D11_MAP_READ,
        D3D11_USAGE_STAGING,
    }
};
use windows::{
    Graphics::{
        Capture::{
            Direct3D11CaptureFramePool,
            GraphicsCaptureSession,
        },
        DirectX::{
            Direct3D11::{
                IDirect3DSurface,
            },
            DirectXPixelFormat
        },
    },
    Win32::{
        Graphics::{
            Direct3D11::{
                ID3D11Device,
                ID3D11DeviceContext,
                ID3D11Texture2D,
                D3D11_TEXTURE2D_DESC,
            },
        },
    }
};

type FrameArrivedHandler =
    windows::Foundation::TypedEventHandler<Direct3D11CaptureFramePool, windows::core::IInspectable>;

use crate::d3d::*;


#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CaptureError {
    // did't init or already closed.
    #[error("Capture is not active.")]
    NotActive,

    // async, so sometimes it's not there.
    #[error("No texture.")]
    NoTexture,

    #[error("CPU read access required.")]
    DeniedAccessCpuRead,

    #[error("Error:")]
    DirectxError(windows::core::Error),

    #[error("Error:")]
    OpencvError(String),

    #[error("Unsupported buffer type. Must be a staging buffer.")]
    UnsupportedBufferType,

    #[error("Unsupported pixel format.")]
    UnsupportedPixelFormat(u32),
}


#[derive(Clone, Debug, Default)]
pub struct RawFrameData {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}


#[derive(Clone, Debug)]
pub struct Capture {
    _d3d_device: ID3D11Device,
    d3d_context: ID3D11DeviceContext,
    frame_pool: Direct3D11CaptureFramePool,
    session: GraphicsCaptureSession,
    _on_frame_arrived: FrameArrivedHandler,
    texture: Arc<Mutex<Option<ID3D11Texture2D>>>,
    active: bool,
}
impl Capture {
    pub fn new(device: &Device) -> anyhow::Result<Self> {
        let d3d_context = Device::get_immediate_context(&device.d3d_device)?;
        let item_size = device.item.Size()?;

        // Initialize the capture
        let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &device.device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            1,
            item_size,
        )?;
        let session = frame_pool.CreateCaptureSession(&device.item)?;

        // to thread safety
        let texture = Arc::new(Mutex::new(None));

        let on_frame_arrived = FrameArrivedHandler::new({
            let d3d_device = device.d3d_device.clone();
            let d3d_context = d3d_context.clone();
            let texture = texture.clone();
            
            move |frame_pool, _| {
                let frame = frame_pool.as_ref().unwrap().TryGetNextFrame()?;
                let surface = frame.Surface()?;

                let frame_texture = Device::from_direct3d_surface(&surface)?;

                // Make a copy of the texture
                let mut desc = D3D11_TEXTURE2D_DESC::default();
                unsafe {
                    frame_texture.GetDesc(&mut desc);
                }
                // Make this a staging texture
                desc.Usage = D3D11_USAGE_STAGING as i32;
                desc.BindFlags = 0;
                desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
                desc.MiscFlags = 0;
                let copy_texture = unsafe {
                    let copy_texture = d3d_device.CreateTexture2D( &desc, std::ptr::null() )?;
                    d3d_context.CopyResource(&copy_texture, &frame_texture);

                    copy_texture
                };

                *texture.lock().unwrap() = Some(copy_texture);

                Ok(())
            }
        });

        // Start the capture
        frame_pool.FrameArrived(on_frame_arrived.clone())?;
        session.StartCapture()?;

        Ok(Self {
            _d3d_device: device.d3d_device.clone(),
            d3d_context,
            frame_pool,
            session,
            _on_frame_arrived: on_frame_arrived,
            texture,
            active: true,
        })
    }

    fn release(&mut self) -> anyhow::Result<()> {
        self.active = false;
    
        // End the capture
        self.session.Close()?;
        self.frame_pool.Close()?;

        Ok(())
    }

    fn take(&self) -> anyhow::Result<IDirect3DSurface, CaptureError> {
        if !self.active {
            return Err(CaptureError::NotActive);
        }
        if self.texture.lock().unwrap().is_none() {
            return Err(CaptureError::NoTexture);
        }

        // Wait for our texture to come
        let surface = Device::to_direct3d_surface(
            self.texture.lock().unwrap().as_ref().unwrap()
        ).map_err(|e| CaptureError::DirectxError(e))?;

        Ok(surface)
    }

    /// rap surface to [RawFrameData]
    fn surface_to_data(&self, surface: &IDirect3DSurface) -> anyhow::Result<RawFrameData, CaptureError> {
        let d3d_texture = Device::from_direct3d_surface(surface).map_err(|e| CaptureError::DirectxError(e))?;

        // Make sure the surface is a pixel format we support
        let desc = unsafe {
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            d3d_texture.GetDesc(&mut desc);

            desc
        };
        let width = desc.Width;
        let height = desc.Height;
        let bytes_per_pixel = match desc.Format {
            DXGI_FORMAT_B8G8R8A8_UNORM => 4,
            _ => return Err(CaptureError::UnsupportedPixelFormat(desc.Format)),
        };

        // TODO: If the texture isn't marked for staging, make a copy
        let d3d_texture = if desc.Usage as u32 == D3D11_USAGE_STAGING {
            if (desc.CPUAccessFlags & D3D11_CPU_ACCESS_READ) == D3D11_CPU_ACCESS_READ {
                d3d_texture
            } else {
                return Err(CaptureError::DeniedAccessCpuRead);
            }
        } else {
            return Err(CaptureError::UnsupportedBufferType);
        };

        // Map the texture
        let mapped = unsafe {
            self.d3d_context.Map(&d3d_texture, 0, D3D11_MAP_READ as i32, 0)
                .map_err(|e| CaptureError::DirectxError(e))?
        };

        // Get a slice of bytes
        let slice: &[u8] = unsafe {
            std::slice::from_raw_parts(
                mapped.pData as *const _,
                (height * mapped.RowPitch) as usize,
            )
        };

        // Make a copy of the data
        let mut data = vec![0u8; ((width * height) * bytes_per_pixel) as usize];
        for row in 0..height {
            let data_begin = (row * (width * bytes_per_pixel)) as usize;
            let data_end = ((row + 1) * (width * bytes_per_pixel)) as usize;
            let slice_begin = (row * mapped.RowPitch) as usize;
            let slice_end = slice_begin + (width * bytes_per_pixel) as usize;
            data[data_begin..data_end].copy_from_slice(&slice[slice_begin..slice_end]);
        }

        // Unmap the texture
        unsafe {
            self.d3d_context.Unmap(&d3d_texture, 0);
        }

        Ok(RawFrameData{
            width: width as i32,
            height: height as i32,
            data
        })
    }

    /// Return rapped current frame with [RawFrameData]
    pub fn get_raw_frame(&self) -> anyhow::Result<RawFrameData, CaptureError> {
        let surface = self.take()?;

        self.surface_to_data(&surface)
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        self.release().unwrap();
    }
}

#[cfg(feature = "img")]
pub mod img;
#[cfg(feature = "img")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "img")))]
pub use img::ImgFrameData;

#[cfg(feature = "mat")]
pub mod mat;
#[cfg(feature = "mat")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "mat")))]
pub use mat::MatFrameData;
