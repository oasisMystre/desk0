use clap::Parser;

mod client;
mod platform;

rust_i18n::i18n!("locales");

#[derive(Parser, Debug)]
struct Args {
  #[arg(long)]
  service: bool,
  #[arg(long)]
  install: bool,
  #[arg(long)]
  uninstall: bool,
  #[arg(long, default_value = "3389")]
  port: u16,
  #[arg(long, default_value = "0.0.0.0")]
  bind_addr: String,
}

pub fn main() {
  #[cfg(target_os = "windows")]
  let locale = crate::platform::windows::locale::get_system_locale();
  #[cfg(not(target_os = "windows"))]
  let locale = "en-US";

  rust_i18n::set_locale(&locale);

  let args = Args::parse();

  #[cfg(target_os = "windows")]
  if args.service {
    if args.install {
      if let Err(error) = crate::platform::windows::service::install() {
        eprintln!("{}: service install error", error)
      }
    } else if args.uninstall {
      if let Err(error) = crate::platform::windows::service::uninstall() {
        eprintln!("{}: service uninstall error", error)
      }
    } else {
      if let Err(error) = crate::platform::windows::service::start() {
        eprintln!("{}: service start error", error)
      }
    }
  }

  #[cfg(not(target_os = "windows"))]
  if args.service {
    eprintln!("Service mode is only supported on Windows.");
  }

  if let Err(error) = run_app() {
    eprintln!("{}: run app error", error);
  }
}

async fn run_app() {
  let rt = tokio::runtime::Runtime::new().expect("failed to build tokio runtime");
  rt.block_on(async {
    let addr = format!("{}:{}", args.bind_addr, args.port);
    println!("Starting Desk0 RDP Agent standalone on RDP://{}", addr);
    if let Err(error) = crate::client::run_server(&addr).await {
      eprintln!("RDP server error: {}", error);
    }
  });
}
