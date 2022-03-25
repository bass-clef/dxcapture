fn main() {
    let device = dxcapture::Device::new_from_window("Code".to_string()).unwrap();
    let capture = dxcapture::Capture::new(&device).unwrap();

    // show desktop example
    while 'q' as i32 != opencv::highgui::wait_key(16).unwrap() {
        let mat = match capture.get_mat_frame() {
            Ok(mat) => mat,
            Err(err) => {
                if err == dxcapture::CaptureError::NoTexture {
                    // async, so sometimes it's not there.
                    continue;
                }
                panic!("{}", err);
            }
        };

        let _ = opencv::highgui::imshow("mat", &mat.data);
    }
}
