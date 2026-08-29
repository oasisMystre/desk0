use enigo::{
  Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard as EnigoKeyboard, Mouse as EnigoMouse,
  Settings,
};
use ironrdp::server::{KeyboardEvent, MouseEvent, RdpServerInputHandler};

pub struct InputHandler {
  enigo: Enigo,
}

unsafe impl Send for InputHandler {}

impl InputHandler {
  pub fn new(settings: &Settings) -> anyhow::Result<Self> {
    let enigo = Enigo::new(settings)?;
    Ok(Self { enigo })
  }
}

impl RdpServerInputHandler for InputHandler {
  fn keyboard(&mut self, event: KeyboardEvent) {
    match event {
      KeyboardEvent::Pressed { code, .. } => {
        let _ = self.enigo.raw(code as u16, Direction::Press);
      }
      KeyboardEvent::Released { code, .. } => {
        let _ = self.enigo.raw(code as u16, Direction::Release);
      }
      KeyboardEvent::UnicodePressed(vk) => {
        if let Some(ch) = char::from_u32(vk as u32) {
          let _ = self.enigo.key(Key::Unicode(ch), Direction::Press);
        }
      }
      KeyboardEvent::UnicodeReleased(vk) => {
        if let Some(ch) = char::from_u32(vk as u32) {
          let _ = self.enigo.key(Key::Unicode(ch), Direction::Release);
        }
      }
      _ => {}
    }
  }

  fn mouse(&mut self, event: MouseEvent) {
    match event {
      MouseEvent::Move { x, y } => {
        let _ = self.enigo.move_mouse(x as i32, y as i32, Coordinate::Abs);
      }
      MouseEvent::LeftPressed => {
        let _ = self.enigo.button(Button::Left, Direction::Press);
      }
      MouseEvent::LeftReleased => {
        let _ = self.enigo.button(Button::Left, Direction::Release);
      }
      MouseEvent::RightPressed => {
        let _ = self.enigo.button(Button::Right, Direction::Press);
      }
      MouseEvent::RightReleased => {
        let _ = self.enigo.button(Button::Right, Direction::Release);
      }
      MouseEvent::MiddlePressed => {
        let _ = self.enigo.button(Button::Middle, Direction::Press);
      }
      MouseEvent::MiddleReleased => {
        let _ = self.enigo.button(Button::Middle, Direction::Release);
      }
      MouseEvent::VerticalScroll { value } => {
        let _ = self.enigo.scroll(value as i32, Axis::Vertical);
      }
      _ => {}
    }
  }
}
