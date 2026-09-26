use std::{num::NonZeroU32, rc::Rc, time::Duration};

use ironrdp::{
  client::rdp::{RdpInputEvent, RdpOutputEvent},
  input::{Database, MousePosition, Operation, Scancode, WheelRotations},
};
use smallvec::SmallVec;
use softbuffer::{Context, Surface};
use tokio::time::Instant;
use winit::{
  application::ApplicationHandler,
  dpi::PhysicalSize,
  event::{MouseScrollDelta, StartCause, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
  keyboard::PhysicalKey,
  window::{CursorIcon, CustomCursor, Window, WindowId},
};

use crate::{
  keymap::{is_modifier, map_key_code},
  viewer::Desk0Viewer,
};

pub enum ViewerEvent {
  Shutdown,
  FrameAvailable,
}

pub enum AppState {
  Initial,
  Suspended {
    window: Rc<Window>,
  },
  Running {
    window: Rc<Window>,
    surface: Surface<OwnedDisplayHandle, Rc<Window>>,
  },
}

pub struct Desk0App {
  inner: Desk0InnerApp,
}

pub struct Desk0InnerApp {
  state: AppState,
  viewer: Desk0Viewer,
  input_database: Database,
  context: Context<OwnedDisplayHandle>,

  // window context
  buffer: Vec<u32>,
  buffer_size: (u16, u16),
  resize_timeout: Option<Instant>,
  last_size: Option<PhysicalSize<u32>>,
}

impl Desk0App {
  pub fn new(
    event_loop: &EventLoop<RdpOutputEvent>,
    viewer: Desk0Viewer,
    initial_size: Option<PhysicalSize<u32>>,
  ) -> anyhow::Result<Self> {
    Ok(Self {
      inner: Desk0InnerApp::new_inner(event_loop, viewer, initial_size)?,
    })
  }
}

impl Desk0InnerApp {
  pub fn new(
    event_loop: &EventLoop<ViewerEvent>,
    viewer: Desk0Viewer,
    initial_size: Option<PhysicalSize<u32>>,
  ) -> anyhow::Result<Self> {
    Self::new_inner(event_loop, viewer, initial_size)
  }

  pub fn new_inner<T: 'static>(
    event_loop: &EventLoop<T>,
    viewer: Desk0Viewer,
    initial_size: Option<PhysicalSize<u32>>,
  ) -> anyhow::Result<Self> {
    let context = Context::new(event_loop.owned_display_handle())
      .map_err(|error| anyhow::anyhow!("unable to initialize softbuffer context: {error}"))?;

    let input_database = Database::new();

    Ok(Self {
      viewer,
      context,
      input_database,
      buffer: vec![],
      resize_timeout: None,
      buffer_size: (0, 0),
      state: AppState::Initial,
      last_size: initial_size,
    })
  }

  fn send_resize_event(&mut self) {
    let Some(size) = self.last_size else {
      return;
    };

    let AppState::Running { window, .. } = &mut self.state else {
      return;
    };

    let scale_factor = (window.scale_factor() * 100.0) as u32;
    let width = u16::try_from(size.width).expect("reasonable width");
    let height = u16::try_from(size.height).expect("reasonable height");

    match self.viewer.input.send(RdpInputEvent::Resize {
      width,
      height,
      scale_factor,
      physical_size: None,
    }) {
      Ok(()) => self.last_size = None,
      Err(error) => {
        self.last_size = None;
        tracing::warn!(
          ?error,
          "unable to enqueue resize event because the RDP session is closed"
        );
      }
    };
  }

  fn handle_viewer_event(&mut self, event_loop: &ActiveEventLoop, event: ViewerEvent) {
    match event {
      ViewerEvent::FrameAvailable => {}
      ViewerEvent::Shutdown => event_loop.exit(),
    }
  }

  pub fn handle_output_event(&mut self, event_loop: &ActiveEventLoop, event: RdpOutputEvent) {
    let AppState::Running { surface, window } = &mut self.state else {
      return;
    };

    match event {
      RdpOutputEvent::ConnectionFailure(error) => {
        tracing::error!(?error, "rdp connection error");
        eprintln!("Connection error: {}", error.report());

        self.viewer.shutdown();
        event_loop.exit();
      }
      RdpOutputEvent::Terminated(graceful_disconnect_reason) => {
        match graceful_disconnect_reason {
          Ok(reason) => {
            tracing::debug!(?reason, "rdp exited");
          }
          Err(error) => {
            //todo: write retry logic here
            tracing::error!(?error, "force exit rdp");
          }
        };

        event_loop.exit();
        self.viewer.shutdown();
      }
      RdpOutputEvent::Image {
        buffer,
        width,
        height,
      } => {
        tracing::trace!(?width, ?height, "Received image with sized");
        tracing::trace!(window_physical_size = ?window.inner_size(), "Drawing image to the window with size");
        self.buffer = buffer;
        self.buffer_size = (width.get(), height.get());

        let window_size = window.inner_size();
        if let (Some(surface_width), Some(surface_height)) = (
          NonZeroU32::new(window_size.width),
          NonZeroU32::new(window_size.height),
        ) {
          surface
            .resize(surface_width, surface_height)
            .expect("surface resize");
        }

        window.request_redraw();
      }
      RdpOutputEvent::PointerHidden => {
        window.set_cursor_visible(false);
      }
      RdpOutputEvent::PointerDefault => {
        window.set_cursor(CursorIcon::default());
        window.set_cursor_visible(true);
      }
      RdpOutputEvent::PointerPosition { x, y } => {
        // Do not warp the local OS cursor from a remote pointer update.
        // set_cursor_position() emits CursorMoved, which is sent back as RDP
        // input and creates a feedback loop that pushes the cursor away.
        tracing::trace!(x, y, "Received remote pointer position");
      }
      RdpOutputEvent::PointerBitmap(pointer) => {
        match CustomCursor::from_rgba(
          pointer.bitmap_data.clone(),
          pointer.width,
          pointer.height,
          pointer.hotspot_x,
          pointer.hotspot_y,
        ) {
          Ok(cursor) => window.set_cursor(event_loop.create_custom_cursor(cursor)),
          Err(error) => tracing::error!(?error, "Failed to set cursor bitmap"),
        }

        window.set_cursor_visible(true);
      }
      _ => {}
    }
  }

  fn on_new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
    if let StartCause::Init = cause {
      let mut window_attributes = Window::default_attributes();
      if let Some(size) = self.last_size {
        window_attributes = window_attributes.with_inner_size(size);
        self.resize_timeout = Some(Instant::now() + Duration::from_secs(1));
      }
      let window = event_loop
        .create_window(window_attributes)
        .expect("failed creating window");
      self.state = AppState::Suspended {
        window: Rc::new(window),
      }
    }
  }

  fn on_about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    if let Some(timeout) = self.resize_timeout {
      if let Some(timeout) = timeout.checked_duration_since(Instant::now()) {
        event_loop.set_control_flow(ControlFlow::wait_duration(timeout));
      } else {
        self.resize_timeout = None;
        self.send_resize_event();
        if let Some(retry_timeout) = self.resize_timeout {
          event_loop.set_control_flow(ControlFlow::wait_duration(
            retry_timeout.saturating_duration_since(Instant::now()),
          ));
        } else {
          event_loop.set_control_flow(ControlFlow::Wait);
        }
      }
    }
  }

  fn on_resumed(&mut self) {
    let AppState::Suspended { window } = &mut self.state else {
      unreachable!("got resumed event while not suspended");
    };
    let mut surface = Surface::new(&self.context, window.clone()).expect("failed creating surface");
    let size = window.inner_size();
    if let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
    {
      if let Err(error) = surface.resize(width, height) {
        tracing::error!(?error, "resizing surface error");
      }
    }

    self.state = AppState::Running {
      surface,
      window: window.clone(),
    };
  }

  fn on_suspended(&mut self, _: &ActiveEventLoop) {
    let AppState::Running { window, .. } = &mut self.state else {
      unreachable!("got resumed event while not runnng");
    };

    let window = window.clone();
    self.state = AppState::Suspended { window };
  }

  fn on_window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    let AppState::Running { window, .. } = &mut self.state else {
      unreachable!("got resumed event while not runnng");
    };

    if window.id() != window_id {
      return;
    }

    match event {
      WindowEvent::Resized(size) => {
        if let AppState::Running { surface, .. } = &mut self.state {
          if let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
          {
            if let Err(error) = surface.resize(width, height) {
              tracing::error!(?error, "failed to resize surface");
            }
          }
        }

        self.last_size = Some(size);
        self.resize_timeout = Some(Instant::now() + Duration::from_secs(1));
      }
      WindowEvent::CloseRequested => {
        self.viewer.shutdown();
        event_loop.exit();
      }
      WindowEvent::KeyboardInput { event, .. } => {
        let key_code = match event.physical_key {
          PhysicalKey::Code(key_code) => key_code,
          PhysicalKey::Unidentified(native_key_code) => {
            tracing::warn!(?native_key_code, "Unsupported physical key, ignored");
            return;
          }
        };

        if is_modifier(key_code) {
          return;
        }

        let Some((scancode, release_only)) = map_key_code(key_code) else {
          tracing::warn!(?key_code, "Unsupported physical key, ignored");
          return;
        };

        let operations: SmallVec<[Operation; 2]> = match event.state {
          winit::event::ElementState::Pressed => {
            smallvec::smallvec![Operation::KeyPressed(scancode)]
          }
          winit::event::ElementState::Released if release_only => {
            smallvec::smallvec![
              Operation::KeyPressed(scancode),
              Operation::KeyReleased(scancode),
            ]
          }
          winit::event::ElementState::Released => {
            smallvec::smallvec![Operation::KeyReleased(scancode),]
          }
        };

        apply_and_send_fast_path_events(&self.viewer, &mut self.input_database, operations);
      }
      WindowEvent::ModifiersChanged(modifiers) => {
        const ALT_LEFT: Scancode = Scancode::from_u8(false, 0x38);
        const LOGO_LEFT: Scancode = Scancode::from_u8(true, 0x5B);
        const SHIFT_LEFT: Scancode = Scancode::from_u8(false, 0x2A);
        const CONTROL_LEFT: Scancode = Scancode::from_u8(false, 0x1D);

        let mut operations = SmallVec::<[Operation; 4]>::new();
        let mut add_operations = |pressed: bool, scancode: Scancode| {
          let operation = if pressed {
            Operation::KeyPressed(scancode)
          } else {
            Operation::KeyReleased(scancode)
          };

          operations.push(operation);
        };

        add_operations(modifiers.state().alt_key(), ALT_LEFT);
        add_operations(modifiers.state().super_key(), LOGO_LEFT);
        add_operations(modifiers.state().shift_key(), SHIFT_LEFT);
        add_operations(modifiers.state().control_key(), CONTROL_LEFT);

        apply_and_send_fast_path_events(&self.viewer, &mut self.input_database, operations);
      }
      WindowEvent::CursorMoved { position, .. } => {
        let window_size = window.inner_size();
        let x = (position.x / f64::from(window_size.width) * f64::from(self.buffer_size.0)) as u16;
        let y = (position.y / f64::from(window_size.height) * f64::from(self.buffer_size.1)) as u16;

        let operation = Operation::MouseMove(MousePosition { x, y });

        apply_and_send_fast_path_events(
          &self.viewer,
          &mut self.input_database,
          core::iter::once(operation),
        );
      }
      WindowEvent::MouseWheel { delta, .. } => {
        let mut operations = SmallVec::<[Operation; 2]>::new();

        match delta {
          MouseScrollDelta::LineDelta(delta_x, delta_y) => {
            if delta_x.abs() > 0.001 {
              operations.push(Operation::WheelRotations(WheelRotations {
                is_vertical: false,
                rotation_units: wheel_units(f64::from(delta_x) * 100.0),
              }));
            }

            if delta_y.abs() > 0.001 {
              operations.push(Operation::WheelRotations(WheelRotations {
                is_vertical: true,
                rotation_units: wheel_units(f64::from(delta_y) * 100.0),
              }));
            }
          }
          MouseScrollDelta::PixelDelta(delta) => {
            if delta.x.abs() > 0.001 {
              operations.push(Operation::WheelRotations(WheelRotations {
                is_vertical: false,
                rotation_units: wheel_units(delta.x),
              }));
            }

            if delta.y.abs() > 0.001 {
              operations.push(Operation::WheelRotations(WheelRotations {
                is_vertical: true,
                rotation_units: wheel_units(delta.y),
              }));
            }
          }
        }

        apply_and_send_fast_path_events(&self.viewer, &mut self.input_database, operations);
      }
      WindowEvent::MouseInput { state, button, .. } => {
        let button = match button {
          winit::event::MouseButton::Back => ironrdp::input::MouseButton::X1,
          winit::event::MouseButton::Forward => ironrdp::input::MouseButton::X2,
          winit::event::MouseButton::Left => ironrdp::input::MouseButton::Left,
          winit::event::MouseButton::Right => ironrdp::input::MouseButton::Right,
          winit::event::MouseButton::Middle => ironrdp::input::MouseButton::Middle,
          winit::event::MouseButton::Other(native_button) => {
            if let Some(button) = ironrdp::input::MouseButton::from_native_button(native_button) {
              button
            } else {
              return;
            }
          }
        };

        let operation = match state {
          winit::event::ElementState::Pressed => Operation::MouseButtonPressed(button),
          winit::event::ElementState::Released => Operation::MouseButtonReleased(button),
        };

        apply_and_send_fast_path_events(
          &self.viewer,
          &mut self.input_database,
          core::iter::once(operation),
        );
      }
      WindowEvent::RedrawRequested => {
        if self.buffer.is_empty() {
          return;
        }

        let AppState::Running { surface, window } = &mut self.state else {
          return;
        };

        let window_size = window.inner_size();
        let destination_width = window_size.width as usize;
        let destination_height = window_size.height as usize;
        let source_width = self.buffer_size.0 as usize;
        let source_height = self.buffer_size.1 as usize;
        let mut buffer = surface.buffer_mut().expect("surface buffer");

        if destination_width == source_width && destination_height == source_height {
          buffer.copy_from_slice(&self.buffer);
        } else if destination_width > 0 && destination_height > 0 {
          for y in 0..destination_height {
            let source_y = y * source_height / destination_height;
            for x in 0..destination_width {
              let source_x = x * source_width / destination_width;
              buffer[y * destination_width + x] = self.buffer[source_y * source_width + source_x];
            }
          }
        }
        buffer.present().expect("buffer present");
      }
      _ => {}
    }
  }
}

impl ApplicationHandler<ViewerEvent> for Desk0InnerApp {
  fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
    self.on_new_events(event_loop, cause);
  }

  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    self.on_about_to_wait(event_loop);
  }

  fn resumed(&mut self, _: &ActiveEventLoop) {
    self.on_resumed();
  }

  fn suspended(&mut self, event_loop: &ActiveEventLoop) {
    self.on_suspended(event_loop);
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    self.on_window_event(event_loop, window_id, event);
  }

  fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ViewerEvent) {
    self.handle_viewer_event(event_loop, event);
  }
}

impl ApplicationHandler<RdpOutputEvent> for Desk0App {
  fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
    self.inner.on_new_events(event_loop, cause);
  }

  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    self.inner.on_about_to_wait(event_loop);
  }

  fn resumed(&mut self, _: &ActiveEventLoop) {
    self.inner.on_resumed();
  }

  fn suspended(&mut self, event_loop: &ActiveEventLoop) {
    self.inner.on_suspended(event_loop);
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    self.inner.on_window_event(event_loop, window_id, event);
  }

  fn user_event(&mut self, event_loop: &ActiveEventLoop, event: RdpOutputEvent) {
    self.inner.handle_output_event(event_loop, event);
  }
}

fn apply_and_send_fast_path_events(
  viewer: &Desk0Viewer,
  input_database: &mut Database,
  operations: impl IntoIterator<Item = Operation>,
) {
  let events = input_database.apply(operations);

  if !events.is_empty() {
    if let Err(error) = viewer.input.send(RdpInputEvent::FastPath(events)) {
      tracing::error!(?error, "rdp input closed");
    }
  }
}

fn wheel_units(value: f64) -> i16 {
  value.round().clamp(-256.0, 255.0) as i16
}
