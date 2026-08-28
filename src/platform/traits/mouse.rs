use ironrdp::server::MouseEvent;

pub trait Mouse {
  fn scroll_wheel(value: u32);
  fn r#move(x: u16, y: u16, width: u16, height: u16);
  fn event(width: u16, height: u16, event: MouseEvent);
}
