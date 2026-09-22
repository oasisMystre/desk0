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
        if let Err(error) = self.enigo.raw(code as u16, Direction::Press) {
          tracing::error!(?error, code, "failed to press keyboard button");
        }
      }
      KeyboardEvent::Released { code, .. } => {
        if let Err(error) = self.enigo.raw(code as u16, Direction::Release) {
          tracing::error!(?error, code, "failed to release keyboard button");
        }
      }
      KeyboardEvent::UnicodePressed(vk) => {
        if let Some(ch) = char::from_u32(vk as u32) {
          if let Err(error) = self.enigo.key(Key::Unicode(ch), Direction::Press) {
            tracing::error!(?error, char = ?ch, "failed to press keyboard unicode button");
          }
        }
      }
      KeyboardEvent::UnicodeReleased(vk) => {
        if let Some(ch) = char::from_u32(vk as u32) {
          if let Err(error) = self.enigo.key(Key::Unicode(ch), Direction::Release) {
            tracing::error!(?error, char = ?ch, "failed to release keyboard unicode button");
          }
        }
      }
      _ => {}
    }
  }

  fn mouse(&mut self, event: MouseEvent) {
    match event {
      MouseEvent::Move { x, y } => {
        if let Err(error) = self.enigo.move_mouse(x as i32, y as i32, Coordinate::Abs) {
          tracing::error!(?error, x, y, "failed to move mouse");
        }
      }
      MouseEvent::LeftPressed => {
        if let Err(error) = self.enigo.button(Button::Left, Direction::Press) {
          tracing::error!(?error, "failed to press left mouse button");
        }
      }
      MouseEvent::LeftReleased => {
        if let Err(error) = self.enigo.button(Button::Left, Direction::Release) {
          tracing::error!(?error, "failed to release left mouse button");
        }
      }
      MouseEvent::RightPressed => {
        if let Err(error) = self.enigo.button(Button::Right, Direction::Press) {
          tracing::error!(?error, "failed to press right mouse button");
        }
      }
      MouseEvent::RightReleased => {
        if let Err(error) = self.enigo.button(Button::Right, Direction::Release) {
          tracing::error!(?error, "failed to release right mouse button");
        }
      }
      MouseEvent::MiddlePressed => {
        if let Err(error) = self.enigo.button(Button::Middle, Direction::Press) {
          tracing::error!(?error, "failed to press middle button");
        }
      }
      MouseEvent::MiddleReleased => {
        if let Err(error) = self.enigo.button(Button::Middle, Direction::Release) {
          tracing::error!(?error, "failed to release middle button");
        }
      }
      MouseEvent::VerticalScroll { value } => {
        if let Err(error) = self.enigo.scroll(value as i32, Axis::Vertical) {
          tracing::error!(?error, "failed to scroll vertical");
        }
      }
      _ => {}
    }
  }
}
