#[repr(C)]
#[derive(Clone)]
pub enum SseTerminalMode {
  Sync = 0,
  Async = 1,
}

#[repr(C)]
#[derive(Clone)]
pub enum SseTerminalInputEvent {
  Poll,
  Create(u32),
  Exit(u32),
  Data {
    id: u32,
    data: Vec<u8>,
    mode: SseTerminalMode,
  },
  Resize {
    id: u32,
    rows: u16,
    cols: u16,
    pw: u16,
    ph: u16,
  },
}

#[repr(C)]
#[repr(i32)]
#[derive(Clone)]
pub enum SseTerminalError {
  SpawnFailed = 0,
  ReadFailed = 1,
  WriteFailed = 2,
  ResizeFailed = 3,
  IoFailed(i32),
}

#[repr(C)]
#[derive(Clone)]
pub enum SseTerminalOutputEvent {
  Connected(u32),
  Data {
    id: u32,
    data: Vec<u8>,
  },
  Exit {
    id: u32,
    code: i32,
    description: Option<String>,
  },
  Error {
    id: u32,
    code: SseTerminalError,
    description: Option<String>,
  },
}

impl Into<pdu::terminal::Mode> for SseTerminalMode {
  fn into(self) -> pdu::terminal::Mode {
    match self {
      SseTerminalMode::Sync => pdu::terminal::Mode::Sync,
      SseTerminalMode::Async => pdu::terminal::Mode::Async,
    }
  }
}

impl Into<pdu::terminal::TerminalInputEvent> for SseTerminalInputEvent {
  fn into(self) -> pdu::terminal::TerminalInputEvent {
    match self {
      Self::Poll => pdu::terminal::TerminalInputEvent::Poll,
      Self::Exit(id) => pdu::terminal::TerminalInputEvent::Exit(id),
      Self::Create(id) => pdu::terminal::TerminalInputEvent::Create(id),
      Self::Data { id, data, mode } => pdu::terminal::TerminalInputEvent::Data {
        id,
        data,
        mode: mode.into(),
      },
      Self::Resize {
        id,
        rows,
        cols,
        pw,
        ph,
      } => pdu::terminal::TerminalInputEvent::Resize {
        id,
        rows,
        cols,
        pw,
        ph,
      },
    }
  }
}

impl From<pdu::terminal::TerminalError> for SseTerminalError {
  fn from(value: pdu::terminal::TerminalError) -> Self {
    match value {
      pdu::terminal::TerminalError::SpawnFailed => Self::SpawnFailed,
      pdu::terminal::TerminalError::ReadFailed => Self::ReadFailed,
      pdu::terminal::TerminalError::WriteFailed => Self::WriteFailed,
      pdu::terminal::TerminalError::ResizeFailed => Self::ResizeFailed,
      pdu::terminal::TerminalError::IoFailed(code) => Self::IoFailed(code),
    }
  }
}

impl From<pdu::terminal::TerminalOutputEvent> for SseTerminalOutputEvent {
  fn from(value: pdu::terminal::TerminalOutputEvent) -> Self {
    match value {
      pdu::terminal::TerminalOutputEvent::Connected(id) => Self::Connected(id),
      pdu::terminal::TerminalOutputEvent::Data { id, data } => Self::Data { id, data },
      pdu::terminal::TerminalOutputEvent::Exit {
        id,
        code,
        description,
      } => Self::Exit {
        id,
        code,
        description,
      },
      pdu::terminal::TerminalOutputEvent::Error {
        id,
        code,
        description,
      } => Self::Error {
        id,
        code: code.into(),
        description,
      },
    }
  }
}
