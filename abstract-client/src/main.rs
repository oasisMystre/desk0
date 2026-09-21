use abstract_client::Viewer;
use softbuffer::{Context, Surface};
use std::{
  num::NonZeroU32,
  rc::Rc,
  sync::{Arc, Mutex},
};
use winit::{
  application::ApplicationHandler,
  event::{ElementState, MouseButton, WindowEvent},
  event_loop::{ActiveEventLoop, EventLoop, OwnedDisplayHandle},
  keyboard::PhysicalKey,
  window::{Window, WindowId},
};

struct App {
  window: Option<Rc<Window>>,
  context: Option<Context<OwnedDisplayHandle>>,
  surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
  viewer: Option<Viewer>,
  frame: Arc<Mutex<(u16, u16, Vec<u8>)>>,
}
impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_some() {
      return;
    }
    let window = Rc::new(
      event_loop
        .create_window(Window::default_attributes().with_title("Abstract Client"))
        .unwrap(),
    );
    let context = Context::new(event_loop.owned_display_handle()).unwrap();
    let surface = Surface::new(&context, window.clone()).unwrap();
    self.surface = Some(surface);
    self.context = Some(context);
    self.window = Some(window);
  }
  fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
    let Some(window) = self.window.as_ref() else {
      return;
    };
    match event {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::RedrawRequested => {
        if let Some(surface) = self.surface.as_mut() {
          let (w, h, rgba) = self.frame.lock().unwrap().clone();
          if let (Some(w), Some(h)) = (NonZeroU32::new(w as u32), NonZeroU32::new(h as u32)) {
            let _ = surface.resize(w, h);
            if let Ok(mut pixels) = surface.buffer_mut() {
              for (dst, src) in pixels.iter_mut().zip(rgba.chunks_exact(4)) {
                *dst = u32::from_le_bytes([src[0], src[1], src[2], 255]);
              }
              let _ = pixels.present();
            }
          }
        }
      }
      WindowEvent::CursorMoved { position, .. } => {
        if let Some(v) = &self.viewer {
          v.mouse_move(position.x as u16, position.y as u16);
        }
      }
      WindowEvent::MouseInput { state, button, .. } => {
        if let Some(v) = &self.viewer {
          let b = match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
            _ => 255,
          };
          if b != 255 {
            v.mouse_button(b, state == ElementState::Pressed, 0, 0);
          }
        }
      }
      WindowEvent::KeyboardInput { event, .. } => {
        if let Some(v) = &self.viewer {
          if let PhysicalKey::Code(code) = event.physical_key {
            if let Some(sc) = scancode(code) {
              v.key(sc, event.state == ElementState::Pressed, is_extended(code));
            }
          }
        }
      }
      _ => {
        window.request_redraw();
      }
    }
  }
}
fn scancode(code: winit::keyboard::KeyCode) -> Option<u8> {
  use winit::keyboard::KeyCode::*;
  Some(match code {
    KeyA => 0x1e,
    KeyB => 0x30,
    KeyC => 0x2e,
    KeyD => 0x20,
    KeyE => 0x12,
    KeyF => 0x21,
    KeyG => 0x22,
    KeyH => 0x23,
    KeyI => 0x17,
    KeyJ => 0x24,
    KeyK => 0x25,
    KeyL => 0x26,
    KeyM => 0x32,
    KeyN => 0x31,
    KeyO => 0x18,
    KeyP => 0x19,
    KeyQ => 0x10,
    KeyR => 0x13,
    KeyS => 0x1f,
    KeyT => 0x14,
    KeyU => 0x16,
    KeyV => 0x2f,
    KeyW => 0x11,
    KeyX => 0x2d,
    KeyY => 0x15,
    KeyZ => 0x2c,
    Digit1 => 0x02,
    Digit2 => 0x03,
    Digit3 => 0x04,
    Digit4 => 0x05,
    Digit5 => 0x06,
    Digit6 => 0x07,
    Digit7 => 0x08,
    Digit8 => 0x09,
    Digit9 => 0x0a,
    Digit0 => 0x0b,
    Enter => 0x1c,
    Escape => 0x01,
    Backspace => 0x0e,
    Tab => 0x0f,
    Space => 0x39,
    ArrowUp => 0x48,
    ArrowDown => 0x50,
    ArrowLeft => 0x4b,
    ArrowRight => 0x4d,
    ShiftLeft => 0x2a,
    ShiftRight => 0x36,
    ControlLeft => 0x1d,
    AltLeft => 0x38,
    F1 => 0x3b,
    F2 => 0x3c,
    F3 => 0x3d,
    F4 => 0x3e,
    F5 => 0x3f,
    F6 => 0x40,
    F7 => 0x41,
    F8 => 0x42,
    F9 => 0x43,
    F10 => 0x44,
    F11 => 0x57,
    F12 => 0x58,
    _ => return None,
  })
}
fn is_extended(code: winit::keyboard::KeyCode) -> bool {
  matches!(
    code,
    winit::keyboard::KeyCode::ArrowUp
      | winit::keyboard::KeyCode::ArrowDown
      | winit::keyboard::KeyCode::ArrowLeft
      | winit::keyboard::KeyCode::ArrowRight
      | winit::keyboard::KeyCode::ControlRight
      | winit::keyboard::KeyCode::AltRight
  )
}
fn main() {
  let event_loop = EventLoop::new().unwrap();
  let frame = Arc::new(Mutex::new((0, 0, Vec::new())));
  let mut app = App {
    window: None,
    context: None,
    surface: None,
    viewer: None,
    frame,
  };
  event_loop.run_app(&mut app).unwrap();
}
