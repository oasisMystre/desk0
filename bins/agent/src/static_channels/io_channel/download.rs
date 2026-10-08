use std::{collections::HashMap, path::Path, str::FromStr};

use futures::StreamExt;
use pdu::io::IoOutputEvent;
use reqwest::Url;
use tokio::{io::AsyncWriteExt, sync::mpsc::UnboundedSender, task::JoinHandle};

#[derive(Debug)]
struct DownloadTask {
  handle: JoinHandle<()>,
}

#[derive(Debug)]
pub struct Download {
  tasks: HashMap<u32, DownloadTask>,
  output_sender: UnboundedSender<IoOutputEvent>,
}

impl Download {
  pub fn new(sender: UnboundedSender<IoOutputEvent>) -> Self {
    Self {
      tasks: HashMap::new(),
      output_sender: sender,
    }
  }

  pub async fn download(&mut self, id: u32, url: &[u8], destination: &[u8]) {
    if let Some(task) = self.tasks.remove(&id) {
      task.handle.abort();
    }

    self.cleanup();

    let url = match Url::from_str(&String::from_utf8_lossy(url)) {
      Ok(url) => url,
      Err(error) => {
        tracing::error!(?error, "parsing url failed");
        let _ = self.output_sender.send(IoOutputEvent::DownloadFailed {
          id,
          description: error.to_string().as_bytes().to_vec(),
        });
        return;
      }
    };

    let destination = Path::new(&String::from_utf8_lossy(destination).to_string()).to_path_buf();

    let handle = tokio::spawn({
      let output_sender = self.output_sender.clone();

      async move {
        let result = async {
          let response = reqwest::get(url).await?.error_for_status()?;

          let mut downloaded = 0u64;
          let total = response.content_length();

          let mut stream = response.bytes_stream();
          let mut file = tokio::fs::File::create(&destination).await?;

          while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            let _ = output_sender.send(IoOutputEvent::DownloadProgress {
              id,
              downloaded,
              total,
            });
          }

          file.flush().await?;

          Ok::<_, anyhow::Error>(())
        }
        .await;

        match result {
          Ok(()) => {
            let _ = output_sender.send(IoOutputEvent::DownloadCompleted {
              id,
              path: destination.to_string_lossy().into_owned().into_bytes(),
            });
          }
          Err(error) => {
            tracing::error!(?id, ?error, "download failed");

            let _ = output_sender.send(IoOutputEvent::DownloadFailed {
              id,
              description: error.to_string().into_bytes(),
            });
          }
        }
      }
    });

    let task = DownloadTask { handle };
    self.tasks.insert(id, task);
  }

  pub fn cleanup(&mut self) {
    self.tasks.retain(|_, task| !task.handle.is_finished());
  }
}
