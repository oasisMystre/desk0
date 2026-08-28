use ironrdp::server::KeyboardEvent;
use windows::Win32::UI::Input::KeyboardAndMouse::{
  INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
  KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY,
};

pub struct Keyboard;

impl Keyboard {
  const INPUT_SIZE: i32 = std::mem::size_of::<INPUT>() as i32;
}

impl crate::platform::traits::Keyboard for Keyboard {
  fn keyevent(code: u8, released: bool, extended: bool) {
    let mut flags = KEYEVENTF_SCANCODE;
    if released {
      flags |= KEYEVENTF_KEYUP;
    }

    if extended {
      flags |= KEYEVENTF_EXTENDEDKEY;
    }

    let input = INPUT {
      r#type: INPUT_KEYBOARD,
      Anonymous: INPUT_0 {
        ki: KEYBDINPUT {
          wVk: VIRTUAL_KEY(0),
          wScan: code as u16,
          dwFlags: flags,
          ..Default::default()
        },
      },
    };

    unsafe {
      SendInput(&[input], Self::INPUT_SIZE);
    }
  }

  fn unicode_keyevent(vk: u16, released: bool) {
    let mut flags = KEYEVENTF_UNICODE;
    if released {
      flags |= KEYEVENTF_KEYUP;
    }

    let input = INPUT {
      r#type: INPUT_KEYBOARD,
      Anonymous: INPUT_0 {
        ki: KEYBDINPUT {
          wVk: VIRTUAL_KEY(0),
          wScan: vk,
          dwFlags: flags,
          ..Default::default()
        },
      },
    };

    unsafe {
      SendInput(&[input], Self::INPUT_SIZE);
    }
  }

  fn event(event: KeyboardEvent) {
    println!("Received keyboard event: {:?}", event);

    match event {
      KeyboardEvent::Pressed { code, extended } => {
        Self::keyevent(code, false, extended);
      }
      KeyboardEvent::Released { code, extended } => {
        Self::keyevent(code, true, extended);
      }
      KeyboardEvent::UnicodePressed(vk) => {
        Self::unicode_keyevent(vk, false);
      }
      KeyboardEvent::UnicodeReleased(vk) => {
        Self::unicode_keyevent(vk, true);
      }
      _ => {}
    }
  }
}
