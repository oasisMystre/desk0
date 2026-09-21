pub mod handlers;
pub mod viewer;

pub use viewer::{ClipboardCallback, FrameCallback, StatusCallback, Viewer, ViewerError};

use std::ffi::{c_char, CStr};

fn string(value: *const c_char) -> Result<String, ()> {
    if value.is_null() { return Err(()); }
    // SAFETY: callers of the C ABI must pass a valid NUL-terminated string.
    unsafe { CStr::from_ptr(value).to_str().map(str::to_owned).map_err(|_| ()) }
}

#[unsafe(no_mangle)]
pub extern "C" fn abstract_viewer_start(
    endpoint: *const c_char,
    token: *const c_char,
    username: *const c_char,
    password: *const c_char,
    on_frame: Option<FrameCallback>,
    on_status: Option<StatusCallback>,
    on_clipboard: Option<ClipboardCallback>,
) -> *mut Viewer {
    let Ok(endpoint) = string(endpoint) else { return std::ptr::null_mut() };
    let Ok(token) = string(token) else { return std::ptr::null_mut() };
    let username = username.is_null().then(String::new).or_else(|| string(username).ok()).unwrap_or_default();
    let password = password.is_null().then(String::new).or_else(|| string(password).ok()).unwrap_or_default();
    match Viewer::start(endpoint, token, username, password, on_frame, on_status, on_clipboard) {
        Ok(viewer) => Box::into_raw(Box::new(viewer)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn abstract_viewer_stop(viewer: *mut Viewer) {
    if !viewer.is_null() {
        // SAFETY: pointer came from abstract_viewer_start and is consumed once.
        drop(unsafe { Box::from_raw(viewer) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn abstract_viewer_mouse_move(viewer: *const Viewer, x: u16, y: u16) {
    if let Some(viewer) = unsafe { viewer.as_ref() } { viewer.mouse_move(x, y); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn abstract_viewer_mouse_button(viewer: *const Viewer, button: u8, pressed: bool, x: u16, y: u16) {
    if let Some(viewer) = unsafe { viewer.as_ref() } { viewer.mouse_button(button, pressed, x, y); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn abstract_viewer_key(viewer: *const Viewer, scancode: u8, pressed: bool, extended: bool) {
    if let Some(viewer) = unsafe { viewer.as_ref() } { viewer.key(scancode, pressed, extended); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn abstract_viewer_set_clipboard_text(viewer: *const Viewer, text: *const c_char) -> bool {
    let Some(viewer) = (unsafe { viewer.as_ref() }) else { return false };
    let Ok(text) = string(text) else { return false };
    viewer.set_clipboard_text(text).is_ok()
}

