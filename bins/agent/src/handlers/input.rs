use enigo::{
  Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard as EnigoKeyboard, Mouse as EnigoMouse,
  Settings,
};
use ironrdp::server::{KeyboardEvent, MouseButton, MouseEvent, RdpServerInputHandler};

pub struct InputHandler {
  enigo: Enigo,
}

unsafe impl Send for InputHandler {}

impl InputHandler {
  pub fn new(settings: &Settings) -> anyhow::Result<Self> {
    let enigo = Enigo::new(settings)?;
    Ok(Self { enigo })
  }

  fn button(button: MouseButton) -> Option<Button> {
    match button {
      MouseButton::Left => Some(Button::Left),
      MouseButton::Right => Some(Button::Right),
      MouseButton::Middle => Some(Button::Middle),
      MouseButton::X1 => Some(Button::Forward),
      MouseButton::X2 => Some(Button::Back),
      _ => None,
    }
  }

  fn mouse_button(&mut self, button: MouseButton, pressed: bool) {
    let Some(button) = Self::button(button) else {
      return;
    };

    let direction = if pressed {
      Direction::Press
    } else {
      Direction::Release
    };

    if let Err(error) = self.enigo.button(button, direction) {
      tracing::error!(?error, ?button, pressed, "failed to send mouse button");
    }
  }

  fn scroll_notches(value: i16) -> i32 {
    let notches = (value as i32) / 120;
    -if notches == 0 {
      value.signum() as i32
    } else {
      notches
    }
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
      MouseEvent::Button {
        button, pressed, ..
      }
      // | MouseEvent::ButtonRel {
      //   button, pressed, ..
      // }
      => {
        self.mouse_button(button, pressed);
      }
      MouseEvent::VerticalScroll { value } => {
        if let Err(error) = self.enigo.scroll(Self::scroll_notches(value), Axis::Vertical) {
          tracing::error!(?error, "failed to scroll vertical");
        }
      }
      MouseEvent::HorizontalScroll { value } => {
        if let Err(error) = self
          .enigo
          .scroll(Self::scroll_notches(value), Axis::Horizontal)
        {
          tracing::error!(?error, "failed to scroll horizontal");
        }
      }
      // MouseEvent::RelMove { x, y } => {
      //   if let Err(error) = self.enigo.move_mouse(x as i32, y as i32, Coordinate::Rel) {
      //     tracing::error!(?error, x, y, "failed to move mouse");
      //   }
      // }
      _ => {}
    }
  }
}
