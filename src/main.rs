use clap::Parser;

use crate::shared::locale::get_system_locale;

mod service;
mod shared;

rust_i18n::i18n!("locales");

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    service: bool,
    #[arg(long)]
    install: bool,
    #[arg(long)]
    uninstall: bool,
}

pub fn main() {
    let locale = get_system_locale();
    rust_i18n::set_locale(&locale);

    let args = Args::parse();

    if args.service {
        if args.install {
            if let Err(error) = service::install() {
                eprintln!("{}: service install error", error)
            }
        } else if args.uninstall {
            if let Err(error) = service::uninstall() {
                eprintln!("{}: service uninstall error", error)
            }
        } else {
            if let Err(error) = service::start() {
                eprintln!("{}: start error", error)
            }
        }
    }
}
