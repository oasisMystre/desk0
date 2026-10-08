use std::{
  fmt::Debug,
  io::{Read, Write},
  process::Output,
};

use pdu::MAX_DATA_SIZE;
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

pub struct Terminal {
  pub reader: Box<dyn Read + Send>,
  pub writer: Box<dyn Write + Send>,
  pub master: Box<dyn MasterPty + Send>,
  pub child: Box<dyn Child + Send>,
}

impl Debug for Terminal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Terminal").finish()
  }
}

impl Terminal {
  pub fn shell<'a>() -> (String, &'a [&'a str]) {
    #[cfg(target_os = "windows")]
    return ("cmd.exe".into(), &["/Q"]);

    #[cfg(target_os = "macos")]
    return ("/bin/zsh".into(), &["-l"]);

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
      let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
      (shell, &["-l"])
    }
  }

  pub fn spawn() -> anyhow::Result<Self> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
      rows: 24,
      cols: 80,
      pixel_width: 0,
      pixel_height: 0,
    })?;

    let (program, args) = Self::shell();
    let mut command = CommandBuilder::new(program);
    command.args(args);

    let child = pair.slave.spawn_command(command)?;

    let reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;

    Ok(Self {
      reader,
      writer,
      child,
      master: pair.master,
    })
  }

  pub fn write<W: Write + Unpin>(w: &mut W, data: &[u8]) -> std::io::Result<()> {
    w.write_all(data)?;
    w.flush()?;
    Ok(())
  }

  pub fn read<R: Read + Unpin>(r: &mut R) -> std::io::Result<Option<Vec<u8>>> {
    let mut buffer = vec![0u8; MAX_DATA_SIZE];
    let n = r.read(&mut buffer)?;
    let result = match n {
      0 => None,
      n => {
        buffer.truncate(n);
        Some(buffer)
      }
    };

    Ok(result)
  }

  pub fn resize(
    m: &mut Box<dyn MasterPty + Send>,
    rows: u16,
    cols: u16,
    pw: u16,
    ph: u16,
  ) -> anyhow::Result<()> {
    m.resize(PtySize {
      rows,
      cols,
      pixel_width: pw,
      pixel_height: ph,
    })
  }

  pub fn run(stdin: &[u8]) -> std::io::Result<Output> {
    let (program, args) = Self::shell();
    let mut child = std::process::Command::new(program)
      .args(args)
      .stdin(std::process::Stdio::piped())
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .spawn()?;

    if let Some(mut writer) = child.stdin.take() {
      writer.write_all(stdin)?;
    }

    child.wait_with_output()
  }
}
