use std::{
  cell::RefCell,
  ffi::{CStr, CString, c_char},
};

use desk0_client::viewer::{Desk0Viewer, Desk0ViewerConfig};
use ironrdp::client::rdp::RdpOutputEvent;

use crate::types::{InputEvent, OutputEvent};

pub type OnOutputEventCallback = extern "C" fn(event: *const OutputEvent);

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(error: impl Into<String>) {
  LAST_ERROR.with(|last_error| {
    *last_error.borrow_mut() = Some(CString::new(error.into()).unwrap());
  });
}

fn string(ptr: *const c_char) -> Option<String> {
  if ptr.is_null() {
    return None;
  }

  unsafe { CStr::from_ptr(ptr).to_str().ok().map(str::to_owned) }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn desk0_create_viewer(
  token: *const c_char,
  endpoint: *const c_char,
  callback: OnOutputEventCallback,
) -> *mut Desk0Viewer {
  let Some(token) = string(token) else {
    set_last_error("invalid token");
    return std::ptr::null_mut();
  };
  let Some(endpoint) = string(endpoint) else {
    set_last_error("invalid endpoint");
    return std::ptr::null_mut();
  };

  let viewer = match Desk0Viewer::with_callback(
    Desk0ViewerConfig {
      token: &token,
      endpoint: &endpoint,
    },
    move |event| {
      let event = match event {
        ironrdp::client::rdp::RdpOutputEvent::Image {
          buffer,
          width,
          height,
        } => {
          let buffer_len = buffer.len();
          let buffer = Box::into_raw(buffer.into_boxed_slice()) as *mut u32;

          OutputEvent::Image {
            buffer,
            buffer_len,
            width: width.into(),
            height: height.into(),
          }
        }
        RdpOutputEvent::PointerDefault => OutputEvent::PointerDefault,
        RdpOutputEvent::PointerHidden => OutputEvent::PointerHidden,
        RdpOutputEvent::PointerPosition { x, y } => OutputEvent::PointerPosition { x, y },
        RdpOutputEvent::PointerBitmap(pointer) => {
          let bitmap_data_len = pointer.bitmap_data.len();
          let bitmap_data =
            Box::into_raw(pointer.bitmap_data.clone().into_boxed_slice()) as *mut u8;

          OutputEvent::PointerBitmap {
            bitmap_data,
            bitmap_data_len,
            width: pointer.width,
            height: pointer.height,
            hotspot_x: pointer.hotspot_x,
            hotspot_y: pointer.hotspot_y,
          }
        }
        RdpOutputEvent::ConnectionFailure(error) => {
          let error = CString::new(error.to_string()).unwrap();
          OutputEvent::ConnectionFailure(error.into_raw())
        }
        RdpOutputEvent::Terminated(_) => OutputEvent::Terminated,
      };

      callback(Box::into_raw(Box::new(event)));
    },
  ) {
    Ok(value) => value,
    Err(error) => {
      set_last_error(error.to_string());
      return std::ptr::null_mut();
    }
  };

  Box::into_raw(Box::new(viewer))
}

#[unsafe(no_mangle)]
pub unsafe fn desk0_free_output_event(event: *mut OutputEvent) -> bool {
  if event.is_null() {
    set_last_error("output event ptr is null");
    return false;
  }

  unsafe {
    match &mut *event {
      OutputEvent::ConnectionFailure(ptr) => {
        if !ptr.is_null() {
          drop(CString::from_raw(*ptr as *mut c_char));
        }
      }
      OutputEvent::Image {
        buffer_len, buffer, ..
      } => {
        if !buffer.is_null() {
          drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            buffer,
            *buffer_len,
          )));
        }
      }
      OutputEvent::PointerBitmap {
        bitmap_data_len,
        bitmap_data,
        ..
      } => {
        if !bitmap_data.is_null() {
          drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            bitmap_data,
            *bitmap_data_len,
          )));
        }
      }
      _ => {}
    }
  };

  true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn desk0_stop_viewer(viewer: *mut Desk0Viewer) -> bool {
  if viewer.is_null() {
    set_last_error("viewer ptr is null");
    return false;
  }

  unsafe {
    drop(Box::from_raw(viewer));
  };

  true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn desk0_send_input_event(
  viewer: *mut Desk0Viewer,
  events: *const InputEvent,
  len: usize,
) -> bool {
  if viewer.is_null() {
    set_last_error("viewer ptr is null");
    return false;
  }

  if events.is_null() {
    set_last_error("events ptr is null");
    return false;
  }

  let viewer = unsafe { &mut *viewer };
  let events = unsafe { std::slice::from_raw_parts(events, len) };

  let operations = events
    .iter()
    .cloned()
    .map(InputEvent::into)
    .collect::<Vec<ironrdp::input::Operation>>();

  viewer.apply_and_send_fast_path_events(operations);

  return true;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn desk0_last_error() -> *const c_char {
  LAST_ERROR.with(|last_error| {
    last_error
      .borrow()
      .as_ref()
      .map_or(std::ptr::null(), |error| error.as_ptr())
  })
}
