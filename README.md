
# dxcapture

`dxcapture` is a library for capturing a Direct3D 11 device on Windows.

```toml
[dependencies]
dxcapture = "1.0"
```

# Details
```rs
let device = dxcapture::Device::default();
let capture = dxcapture::Capture::new(&device).unwrap();

let raw = loop {
    match capture.get_raw_frame() {
        Ok(raw) => break raw,
        Err(e) => {
            if e == dxcapture::CaptureError::NoTexture {
                // async, so sometimes it's not there.
                continue;
            }
            panic!("{}", e);
        }
    }
};

// taked primary monitor.
// hoge raw
```

## Optional Features
- *`img`* - Enable features that depend on the [`image`](https://docs.rs/image/) crate
    ```toml
    dxcapture = { version = "1.0", features = ["img"] }
    ```
    ```rs
    let device = dxcapture::Device::default();
    let capture = dxcapture::Capture::new(&device).unwrap();
    
    let image = capture.wait_img_frame().expect("Failed to capture");
    let path = "image.png";
    
    image.data.save(path).expect("Failed to save");
    ```
    [Read more with image](`Capture::get_img_frame`)

- *`mat`* - Enable features that depend on the [`opencv`](https://docs.rs/opencv/) crate
    ```toml
    dxcapture = { version = "1.0", features = ["mat"] }
    ```
    ```rs
    use opencv::prelude::*;
    use opencv::imgcodecs::{ imwrite, IMWRITE_PNG_STRATEGY_DEFAULT };
    
    let device = dxcapture::Device::default();
    let capture = dxcapture::Capture::new(&device).unwrap();
    
    let mat = capture.wait_mat_frame().expect("Failed to capture");
    let path = "image.png";
    
    imwrite(path, &mat.data, &vec![IMWRITE_PNG_STRATEGY_DEFAULT].into()).expect("Failed to save");
    ```
    [Read more with opencv](`Capture::get_mat_frame`)

## Exmaples
- [examples](examples/)

# Documentation
- [docs.rs](https://docs.rs/dxcapture/)

# License
- [MIT License](LICENSE) (http://opensource.org/licenses/MIT)

# Special Thanks
This crate completed thanks to [wgc-rust-demo](https://github.com/robmikh/wgc-rust-demo)
