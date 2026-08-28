use ironrdp::server::KeyboardEvent;

pub trait Keyboard {
  fn keyevent(code: u8, released: bool, extended: bool);
  fn unicode_keyevent(vk: u16, released: bool);

  fn event(event: KeyboardEvent);
}
