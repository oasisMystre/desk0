use std::process::Stdio;

use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
};

#[derive(Debug)]
pub struct Terminal {
  pub stdin: ChildStdin,
  pub stdout: ChildStdout,
  pub stderr: ChildStderr,
}

impl Terminal {
  fn new(mut child: Child) -> std::io::Result<Self> {
    let missing = |key: &'static str| {
      std::io::Error::new(
        std::io::ErrorKind::BrokenPipe,
        format!("failed to open {key}"),
      )
    };
    let stdin = child.stdin.take().ok_or_else(|| missing("stdin"))?;
    let stdout = child.stdout.take().ok_or_else(|| missing("stdout"))?;
    let stderr = child.stderr.take().ok_or_else(|| missing("stderr"))?;

    Ok(Self {
      stdin,
      stdout,
      stderr,
    })
  }

  fn shell() -> Command {
    #[cfg(target_os = "windows")]
    let command = Command::new("cmd.exe").arg("/Q");

    #[cfg(target_os = "macos")]
    let command = {
      let mut command = Command::new("/bin/zsh");
      command.arg("-l");
      command
    };

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let command = {
      let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
      let mut command = Command::new(shell);
      command.arg("-l");
      command
    };

    command
  }

  pub fn spawn() -> std::io::Result<Self> {
    let child = Self::shell()
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .kill_on_drop(true)
      .spawn()?;

    Ok(Self::new(child)?)
  }

  pub async fn write<W: AsyncWriteExt + Unpin>(w: &mut W, data: &[u8]) -> std::io::Result<()> {
    w.write_all(data).await?;
    w.flush().await?;
    Ok(())
  }

  pub async fn read<R: AsyncReadExt + Unpin>(r: &mut R) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![0u8; 16 * 1024];
    let n = r.read(&mut buffer).await?;

    buffer.truncate(n);
    Ok(buffer)
  }
}
