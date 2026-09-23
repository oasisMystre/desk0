use clap::Parser;
use rfd::{MessageButtons, MessageDialog, MessageLevel};
use rust_i18n::t;
use tokio_util::sync::CancellationToken;

use crate::api::{Api, ApiConfig};

mod api;
mod handlers;
mod platform;
mod server;

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

  /// Central Desk0 server address to connect outbound reverse proxy tunnel to
  #[arg(long, default_value = "ws://172.20.10.4:8080")]
  server_addr: String,

  #[arg(long, default_value = "http:///172.20.10.4:8080")]
  api_addr: String,

  /// Organization ID / fingerprint override
  #[arg(long, default_value = "11111111-1111-1111-1111-111111111111")]
  org_id: String,

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
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
          tracing_subscriber::EnvFilter::new(
            "info,desk0_agent=debug,copypaste=debug,ironrdp=debug,ironrdp_cliprdr=debug",
          )
        }),
    )
    .with_target(true)
    .with_thread_names(true)
    .init();

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
        eprintln!("Service install error: {error:?}");
      }
      return;
    }

    if args.uninstall {
      if let Err(error) = crate::platform::windows::service::uninstall() {
        eprintln!("Service uninstall error: {error:?}");
      }
      return;
    }

    if args.service {
      if let Err(error) = crate::platform::windows::service::start() {
        eprintln!("Service start error: {error:?}");
      }
      return;
    }
  }

  #[cfg(not(target_os = "windows"))]
  if args.service || args.install || args.uninstall {
    eprintln!("Service installation mode is currently supported on Windows.");
    return;
  }

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("failed to build tokio multi-thread runtime");

  let local = tokio::task::LocalSet::new();

  local.block_on(&rt, async {
    let cancellation_token = CancellationToken::new();
    let cancellation_token_clone = cancellation_token.clone();

    tokio::select! {
      _ = tokio::signal::ctrl_c() => {
        println!("Received shutdown signal. Stopping Desk0 Agent...");
        cancellation_token.cancel();
      }
      result = async {
        rustls::crypto::ring::default_provider()
            .install_default()
            .unwrap();

        if args.listen {
          use crate::server::traits::Server;

          let addr = format!("{}:{}", args.bind_addr, args.port);
          println!("Starting Desk0 RDP Agent standalone on RDP://{}", addr);
          let server = crate::server::server::AgentServer::new(addr.parse()?);
          server.run(cancellation_token_clone).await?;
        } else {
          use crate::server::traits::Tunnel;

          let api = Api::from_config(ApiConfig{base_url: &args.api_addr});
          let db = rocksdb::DB::open_default("data")?;

          let identity = crate::server::identity::Identity::upsert(&api, &db, args.org_id).await?;
          let tunnel = crate::server::tunnel::AgentTunnel::new(args.server_addr, identity, api);
          tunnel.run( cancellation_token_clone).await?;
        }
        Ok::<_, anyhow::Error>(())
      } => {
        if let Err(error) = result {
          let _ = MessageDialog::new()
              .set_title(t!("error_title"))
              .set_description(t!("error_description"))
              .set_level(MessageLevel::Error)
              .set_buttons(MessageButtons::Ok).show();

            eprintln!("Desk0 agent error: {error:#?}");
        }
      }
    }
  });
}
