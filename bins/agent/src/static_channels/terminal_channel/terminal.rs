use std::{
  fmt::Debug,
  io::{Read, Write},
};

use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};

pub struct Terminal {
  pub master: Box<dyn MasterPty + Send>,
  pub reader: Box<dyn Read + Send>,
  pub writer: Box<dyn Write + Send>,
}

impl Debug for Terminal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Terminal").finish()
  }
}

impl Terminal {
  fn shell() -> CommandBuilder {
    #[cfg(target_os = "windows")]
    let command = {
      let mut command = CommandBuilder::new("cmd.exe");
      command.arg("/Q");
      command
    };

    #[cfg(target_os = "macos")]
    let command = {
      let mut command = CommandBuilder::new("/bin/zsh");
      command.arg("-l");
      command
    };

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let command = {
      let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
      let mut command = CommandBuilder::new(shell);
      command.arg("-l");
      command
    };

    command
  }

  pub fn spawn() -> anyhow::Result<Self> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
      rows: 24,
      cols: 80,
      pixel_width: 0,
      pixel_height: 0,
    })?;

    let command = Self::shell();
    pair.slave.spawn_command(command)?;

    let reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;

    Ok(Self {
      reader,
      writer,
      master: pair.master,
    })
  }

  pub async fn write<W: Write + Unpin>(w: &mut W, data: &[u8]) -> std::io::Result<()> {
    w.write_all(data)?;
    w.flush()?;
    Ok(())
  }

  pub async fn read<R: Read + Unpin>(r: &mut R) -> std::io::Result<Option<Vec<u8>>> {
    let mut buffer = vec![0u8; 16 * 1024];
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
}
