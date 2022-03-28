/* 
 * author: Humi@bass_clef_ <bassclef.nico@gmail.com>
 */

#![cfg_attr(feature = "docs-features", feature(doc_cfg))]

//! # dxcapture
//! `dxcapture` is a library for capturing a Direct3D 11 device on Windows.
//! 
//! # Examples
//! ```
//! let device = dxcapture::Device::default();
//! let capture = dxcapture::Capture::new(&device).unwrap();
//! 
//! let raw = loop {
//!     match capture.get_raw_frame() {
//!         Ok(raw) => break raw,
//!         Err(e) => {
//!             if e == dxcapture::CaptureError::NoTexture {
//!                 // async, so sometimes it's not there.
//!                 continue;
//!             }
//!             panic!("{}", e);
//!         }
//!     }
//! };
//! // taked primary monitor.
//! // hoge raw
//! ```
//! 
//! [Read more with image](`Capture::get_img_frame`)
//! 
//! [Read more with opencv](`Capture::get_mat_frame`)

pub mod d3d;
pub mod capture;

pub use d3d::*;
pub use capture::*;

mod displays;
mod window_finder;

pub use displays::enumerate_displays as enumerate_displays;
pub use window_finder::get_capturable_windows as enumerate_windows;
