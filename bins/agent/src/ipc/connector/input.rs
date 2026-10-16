use enigo::{
  Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard as EnigoKeyboard, Mouse as EnigoMouse,
  Settings,
};
use ipc_pdu::inbound::{IpcInputEvent, IpcKeyboardEvent, IpcMouseButton, IpcMouseEvent};
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tokio_util::sync::CancellationToken;

struct IpcInput {
  enigo: Enigo,
  vertical_wheel_remainder: i32,
  horizontal_wheel_remainder: i32,
}

pub struct IpcInputHandler {
  settings: Settings,
  handle: Option<JoinHandle<()>>,
  cancellation_token: CancellationToken,
}

impl IpcInput {
  fn button(button: IpcMouseButton) -> Option<Button> {
    match button {
      IpcMouseButton::Left => Some(Button::Left),
      IpcMouseButton::Right => Some(Button::Right),
      IpcMouseButton::Middle => Some(Button::Middle),
      IpcMouseButton::X1 => Some(Button::Forward),
      IpcMouseButton::X2 => Some(Button::Back),
      _ => None,
    }
  }

  fn mouse_button(&mut self, button: IpcMouseButton, pressed: bool) {
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

  fn scroll_notches(remainder: &mut i32, value: i16) -> i32 {
    let total = *remainder + i32::from(value);
    let notches = total / 120;
    *remainder = total - notches * 120;
    notches
  }

  fn keyboard(&mut self, event: IpcKeyboardEvent) {
    match event {
      IpcKeyboardEvent::Pressed { code, .. } => {
        if let Err(error) = self.enigo.raw(code as u16, Direction::Press) {
          tracing::error!(?error, code, "failed to press keyboard button");
        }
      }
      IpcKeyboardEvent::Released { code, .. } => {
        if let Err(error) = self.enigo.raw(code as u16, Direction::Release) {
          tracing::error!(?error, code, "failed to release keyboard button");
        }
      }
      IpcKeyboardEvent::UnicodePressed(vk) => {
        if let Some(ch) = char::from_u32(vk as u32) {
          if let Err(error) = self.enigo.key(Key::Unicode(ch), Direction::Press) {
            tracing::error!(?error, char = ?ch, "failed to press keyboard unicode button");
          }
        }
      }
      IpcKeyboardEvent::UnicodeReleased(vk) => {
        if let Some(ch) = char::from_u32(vk as u32) {
          if let Err(error) = self.enigo.key(Key::Unicode(ch), Direction::Release) {
            tracing::error!(?error, char = ?ch, "failed to release keyboard unicode button");
          }
        }
      }
      _ => {}
    }
  }

  fn mouse(&mut self, event: IpcMouseEvent) {
    match event {
      IpcMouseEvent::Move { x, y } => {
        if let Err(error) = self.enigo.move_mouse(x as i32, y as i32, Coordinate::Abs) {
          tracing::error!(?error, x, y, "failed to move mouse");
        }
      }
      IpcMouseEvent::RelMove { x, y } => {
        if let Err(error) = self.enigo.move_mouse(x as i32, y as i32, Coordinate::Rel) {
          tracing::error!(?error, x, y, "failed to move mouse");
        }
      }
      IpcMouseEvent::Button {
        button, pressed, ..
      }
      | IpcMouseEvent::ButtonRel {
        button, pressed, ..
      } => {
        self.mouse_button(button, pressed);
      }
      IpcMouseEvent::VerticalScroll { value } => {
        let notches = Self::scroll_notches(&mut self.vertical_wheel_remainder, value);
        if notches != 0 {
          if let Err(error) = self.enigo.scroll(notches, Axis::Vertical) {
            tracing::error!(?error, ?notches, "failed to scroll vertical");
          }
        }
      }
      IpcMouseEvent::HorizontalScroll { value } => {
        let notches = Self::scroll_notches(&mut self.horizontal_wheel_remainder, value);
        if notches != 0 {
          if let Err(error) = self.enigo.scroll(notches, Axis::Horizontal) {
            tracing::error!(?error, ?notches, "failed to scroll horizontal");
          }
        }
      }
      _ => {}
    }
  }
}

impl IpcInputHandler {
  pub fn new(settings: Settings, cancellation_token: CancellationToken) -> Self {
    Self {
      settings,
      handle: None,
      cancellation_token,
    }
  }

  pub fn start(&mut self, mut receiver: UnboundedReceiver<IpcInputEvent>) -> anyhow::Result<()> {
    let cancellation_token = self.cancellation_token.clone();
    let mut input = IpcInput {
      vertical_wheel_remainder: 0,
      horizontal_wheel_remainder: 0,
      enigo: Enigo::new(&self.settings)?,
    };

    let handle = tokio::task::spawn_local(async move {
      loop {
        let result = tokio::select! {
            _ = cancellation_token.cancelled() => break,
            result = receiver.recv() => result
        };

        let Some(event) = result else {
          break;
        };

        if cancellation_token.is_cancelled() {
          break;
        }

        match event {
          IpcInputEvent::Mouse(event) => input.mouse(event.into()),
          IpcInputEvent::Keyboard(event) => input.keyboard(event.into()),
          IpcInputEvent::Stop => cancellation_token.cancel(),
        }
      }
    });

    self.handle = Some(handle);

    Ok(())
  }

  #[allow(dead_code)]
  pub async fn stop(&mut self) {
    self.cancellation_token.cancel();

    if let Some(handle) = self.handle.take() {
      if let Err(error) = handle.await {
        tracing::error!(?error, "input handler task failed");
      }
    }
  }
}
