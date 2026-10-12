#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use clap::Parser;
use rfd::{MessageButtons, MessageDialog, MessageLevel};
use rust_i18n::t;
use shared::init::init_tracing;
use tokio_util::sync::CancellationToken;

use crate::{
  api::{Api, ApiConfig},
  config::AgentConfig,
};

mod api;
mod config;
mod handlers;
mod platform;
mod server;
mod static_channels;

rust_i18n::i18n!("locales");

#[derive(Parser, Debug)]
#[command(name = "desk0-agent", about = "Desk0 Client Agent")]
struct Args {
  /// Run as a Windows Service (invoked by Windows Service Controller)
  #[arg(long)]
  service: bool,

  /// Request elevation and install agent as a background Windows Service
  #[arg(long)]
  install: bool,

  /// Request elevation and uninstall the background Windows Service
  #[arg(long)]
  uninstall: bool,

  /// Run without GUI error dialogs Used by the background service
  #[arg(long)]
  headless: bool,

  /// Run standalone direct RDP listener instead of connecting outbound to Desk0 Server
  #[arg(long)]
  listen: bool,

  /// Port for standalone direct listener
  #[arg(long, default_value = "3389")]
  port: u16,

  /// Bind address for standalone direct listener
  #[arg(long, default_value = "0.0.0.0")]
  bind_addr: String,
}

const SUPPORTED_LOCALES: [&str; 1] = ["en-US"];

pub fn main() {
  if let Err(error) = init_tracing() {
    eprintln!("Failed to initialize tracing: {error:#}");
  }

  rust_i18n::i18n!("locale", fallback = ["en-US"]);

  let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
  let locale = if SUPPORTED_LOCALES.contains(&locale.as_str()) {
    &locale
  } else {
    "en-US"
  };
  rust_i18n::set_locale(locale);

  let args = Args::parse();

  #[cfg(target_os = "windows")]
  {
    if args.install {
      if let Err(error) = crate::platform::windows::service::install() {
        tracing::error!(?error, "Service install error");
      }
      return;
    }

    if args.uninstall {
      if let Err(error) = crate::platform::windows::service::uninstall() {
        tracing::error!(?error, "Service uninstall error");
      }
      return;
    }

    if args.service {
      if let Err(error) = crate::platform::windows::service::start() {
        tracing::error!(?error, "Service start error");
      }
      return;
    }
  }

  #[cfg(not(target_os = "windows"))]
  if args.service || args.install || args.uninstall {
    tracing::error!("Service installation mode is currently supported on Windows.");
    return;
  }

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("failed to build tokio multi-thread runtime");

  let local = tokio::task::LocalSet::new();

  local.block_on(&rt, async {
    let cancellation_token = CancellationToken::new();

    let run_cancellation_token = cancellation_token.clone();

    let run = async {
      rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();

      if args.listen {
        use crate::server::traits::Server;

        let addr = format!("{}:{}", args.bind_addr, args.port);
        tracing::info!("Starting Desk0 RDP Agent standalone on RDP://{}", addr);
        let server = crate::server::server::AgentServer::new(addr.parse()?);
        server.run(run_cancellation_token).await?;
      } else {
        use crate::server::traits::Tunnel;

        let config = AgentConfig::load_config()?;
        let api = Api::from_config(ApiConfig {
          base_url: &config.api_url,
        });

        let identity = crate::server::identity::Identity::upsert(&api, config.organization).await?;
        let tunnel = crate::server::tunnel::AgentTunnel::new(config.tunnel_url, identity, api);
        tunnel.run(run_cancellation_token).await?;
      }
      Ok::<_, anyhow::Error>(())
    };

    tokio::pin!(run);
    tokio::select! {
      _ = tokio::signal::ctrl_c() => {
        tracing::debug!("Received shutdown signal. Stopping Desk0 Agent.");
        cancellation_token.cancel();

        if let Err(error) = (&mut run).await {
            tracing::error!(?error, "Desk0 agent shutdown error");
        }
      }
      result = &mut run => {
        if let Err(error) = result {
         if !args.headless {
             let _ = MessageDialog::new()
                 .set_title(t!("error_title"))
                 .set_description(format!("{}\n\n{}", t!("error_description"), error.to_string()))
                 .set_level(MessageLevel::Error)
                 .set_buttons(MessageButtons::Ok).show();
         }

            tracing::error!(?error, "Desk0 agent error");
        }
      }
    }
  });
}
