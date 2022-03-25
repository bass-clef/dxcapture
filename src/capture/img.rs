use image::{
    DynamicImage,
    ImageBuffer,
    RgbaImage,
    Bgra,
};

use super::*;

#[derive(Clone, Debug, Default)]
/// this is container for image.
/// 
/// [Read more](`Capture::get_img_frame`)
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "img")))]
pub struct ImgFrameData {
    pub width: i32,
    pub height: i32,
    pub data: RgbaImage,
}
impl ImgFrameData {
    pub fn new(width: i32, height: i32, data: RgbaImage) -> Self {
        Self{
            width, height, data
        }
    }
}

impl Capture {
    /// Get image RgbaImage from a Direct3D surface
    /// 
    /// for [image] crate.
    /// 
    /// Required features: *`"img"`*
    /// # Examples
    /// ```
    /// let device = dxcapture::Device::default();
    /// let capture = dxcapture::Capture::new(&device).unwrap();
    /// 
    /// let image = capture.wait_img_frame().expect("Failed to capture");
    /// let path = "image.png";
    /// 
    /// image.data.save(path).expect("Failed to save");
    /// ```
    pub fn get_img_frame(&self) -> anyhow::Result<ImgFrameData, CaptureError> {
        let raw = self.get_raw_frame()?;

        let image: ImageBuffer<Bgra<u8>, _> =
            ImageBuffer::from_raw(raw.width as u32, raw.height as u32, raw.data).unwrap();
        let dynamic_image = DynamicImage::ImageBgra8(image);
        let dynamic_image = dynamic_image.to_rgba8();

        Ok(ImgFrameData::new( raw.width as i32, raw.height as i32, dynamic_image ))
    }

    /// Get opencv image from a Direct3D surface. with throught NoTexture
    pub fn wait_img_frame(&self) -> anyhow::Result<ImgFrameData, CaptureError> {
        loop {
            match self.get_img_frame() {
                Ok(mat) => return Ok(mat),
                Err(e) => {
                    if e == CaptureError::NoTexture {
                        continue;
                    }
                    return Err(e);
                },
            }
        }
    }
}
