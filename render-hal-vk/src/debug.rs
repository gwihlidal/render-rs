#![allow(dead_code)]
#![allow(unused_variables)]

/// Raw mutable pointer to the OS-provided window handle.
//pub type WindowHandle = *const c_void;

#[cfg(target_os = "windows")]
fn rd_test() {
    use renderdoc::{RenderDoc, V110};

    let mut render_doc: RenderDoc<V110> =
        RenderDoc::new().expect("Failed to initialize RenderDoc v110");

    let (major, minor, patch) = render_doc.get_api_version();
    assert_eq!(major, 1u32);
    assert_eq!(minor, 1u32);

    //render_doc.set_active_window(window.context(), ::std::ptr::null());
    //render_doc.set_focus_toggle_keys(&[glutin::VirtualKeyCode::F]);
    //render_doc.set_capture_keys(&[glutin::VirtualKeyCode::C]);
    //render_doc.mask_overlay_bits(OverlayBits::DEFAULT, OverlayBits::DEFAULT);

    // When a certain key is pressed, trigger a single-frame capture like this.
    render_doc.trigger_capture();

    // If you specify version `V110` or newer, you can trigger a multi-frame
    // capture like this.
    render_doc.trigger_multi_frame_capture(3);

    // Query the details of an existing capture like this.
    match render_doc.get_capture(0) {
        Some(cap) => trace!("ID: 0, Path: {}, Timestamp: {}", cap.0, cap.1),
        None => trace!("No capture found with ID of 0!"),
    }

    //match rd.launch_replay_ui(None) {
    //	Ok(pid) => trace!("Launched replay UI ({}).", pid),
    //	Err(err) => trace!("{:?}", err),
    //}
}
