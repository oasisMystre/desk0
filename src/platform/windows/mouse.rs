use KeyboardAndMouse::{
  INPUT, INPUT_0, INPUT_MOUSE, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_MOVE,
  MOUSEEVENTF_WHEEL, MOUSEINPUT, SendInput,
};
use ironrdp::server::MouseEvent;
use windows::Win32::UI::Input::KeyboardAndMouse;

pub struct Mouse;

impl Mouse {
  const MOUSE_COORDINATE_MAX: u32 = u16::MAX as u32;
  const INPUT_SIZE: i32 = std::mem::size_of::<INPUT>() as i32;

  fn set_button_flag(button_flag: MOUSE_EVENT_FLAGS) {
    let input = INPUT {
      r#type: INPUT_MOUSE,
      Anonymous: INPUT_0 {
        mi: MOUSEINPUT {
          dwFlags: button_flag,
          ..Default::default()
        },
      },
    };

    unsafe {
      SendInput(&[input], Self::INPUT_SIZE);
    }
  }
}

impl crate::platform::traits::Mouse for Mouse {
  fn r#move(x: u16, y: u16, width: u16, height: u16) {
    let dx = (x as u32 * Self::MOUSE_COORDINATE_MAX / width as u32).max(1) as i32;
    let dy = (y as u32 * Self::MOUSE_COORDINATE_MAX / height as u32).max(1) as i32;

    let input = INPUT {
      r#type: INPUT_MOUSE,
      Anonymous: INPUT_0 {
        mi: MOUSEINPUT {
          dx,
          dy,
          dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
          ..Default::default()
        },
      },
    };

    unsafe {
      SendInput(&[input], Self::INPUT_SIZE);
    }
  }

  fn scroll_wheel(value: u32) {
    let input = INPUT {
      r#type: INPUT_MOUSE,
      Anonymous: INPUT_0 {
        mi: MOUSEINPUT {
          mouseData: value,
          dwFlags: MOUSEEVENTF_WHEEL,
          ..Default::default()
        },
      },
    };

    unsafe {
      SendInput(&[input], Self::INPUT_SIZE);
    }
  }

  fn event(width: u16, height: u16, event: MouseEvent) {
    match event {
      MouseEvent::Move { x, y } => {
        Self::r#move(x, y, width, height);
      }
      MouseEvent::LeftPressed => {
        Self::set_button_flag(KeyboardAndMouse::MOUSEEVENTF_LEFTDOWN);
      }
      MouseEvent::LeftReleased => {
        Self::set_button_flag(KeyboardAndMouse::MOUSEEVENTF_LEFTUP);
      }
      MouseEvent::RightPressed => {
        Self::set_button_flag(KeyboardAndMouse::MOUSEEVENTF_RIGHTDOWN);
      }
      MouseEvent::RightReleased => {
        Self::set_button_flag(KeyboardAndMouse::MOUSEEVENTF_RIGHTUP);
      }
      MouseEvent::MiddlePressed => {
        Self::set_button_flag(KeyboardAndMouse::MOUSEEVENTF_MIDDLEDOWN);
      }
      MouseEvent::MiddleReleased => {
        Self::set_button_flag(KeyboardAndMouse::MOUSEEVENTF_MIDDLEUP);
      }
      MouseEvent::VerticalScroll { value } => {
        Self::scroll_wheel(value as u32);
      }
      _ => {}
    }
  }
}
