use opencv::core;
use winapi::shared::{
    minwindef::LPVOID,
};

use super::*;

#[derive(Clone, Debug, Default)]
/// this is container for opencv.
/// 
/// [Read more](`Capture::get_mat_frame`)
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "mat")))]
pub struct MatFrameData {
    pub width: i32,
    pub height: i32,
    pub data: opencv::core::Mat,
    _raw_data: Vec<u8>,
}
impl MatFrameData {
    pub fn new(width: i32, height: i32, data: core::Mat, raw_data: Vec<u8>) -> Self {
        Self{
            width, height, data, _raw_data: raw_data
        }
    }
}

impl Capture {
    /// Get opencv Mat from a Direct3D surface
    /// 
    /// for [opencv] crate.
    /// 
    /// Required features: *`"mat"`*
    /// # Examples
    /// ```
    /// use opencv::prelude::*;
    /// use opencv::imgcodecs::{ imwrite, IMWRITE_PNG_STRATEGY_DEFAULT };
    /// 
    /// let device = dxcapture::Device::default();
    /// let capture = dxcapture::Capture::new(&device).unwrap();
    /// 
    /// let mat = capture.wait_mat_frame().expect("Failed to capture");
    /// let path = "image.png";
    /// 
    /// imwrite(path, &mat.data, &vec![IMWRITE_PNG_STRATEGY_DEFAULT].into()).expect("Failed to save");
    /// ```
    pub fn get_mat_frame(&self) -> anyhow::Result<MatFrameData, CaptureError> {
        let raw = self.get_raw_frame()?;

        let mat_data = unsafe {
            core::Mat::new_rows_cols_with_data(
                raw.height as i32, raw.width as i32, core::CV_8UC4,
                raw.data.as_ptr() as LPVOID, core::Mat_AUTO_STEP
            ).map_err(|err| CaptureError::OpencvError(err.to_string()))?
        };

        Ok(MatFrameData::new( raw.width as i32, raw.height as i32, mat_data, raw.data ))
    }

    /// Get opencv Mat from a Direct3D surface. with throught NoTexture
    pub fn wait_mat_frame(&self) -> anyhow::Result<MatFrameData, CaptureError> {
        loop {
            match self.get_mat_frame() {
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
