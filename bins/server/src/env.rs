use strum::Display;
use shared::env::FromEnv;

#[derive(Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Env {
  AuthToken,
}

impl FromEnv for Env {}
