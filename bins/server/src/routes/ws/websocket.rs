use axum::{
  extract::{Query, State, WebSocketUpgrade},
  http::HeaderMap,
  response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::routes::{
  state::AppState,
  ws::{agent, context::WsContext, rdp},
};

#[derive(Deserialize)]
pub struct WebSocketQueryParams {
  pub token: Option<String>,
}

pub async fn ws_route(
  State(AppState {
    ws_context: WsContext {
      auth,
      registry,
      certificate,
    },
    ..
  }): State<AppState>,
  ws: WebSocketUpgrade,
  headers: HeaderMap,
  Query(WebSocketQueryParams { token }): Query<WebSocketQueryParams>,
) -> impl IntoResponse {
  let token = match token {
    Some(token) => token,
    None => {
      let Some(token) = headers
        .get("rdcleanpath-token")
        .and_then(|token| token.to_str().ok())
      else {
        return Response::builder()
          .status(401)
          .body("Unauthorized".into())
          .unwrap();
      };

      token.to_string()
    }
  };

  match auth.authenticate(&token) {
    Ok(claims) => match claims.role.as_deref() {
      Some("agent") => ws.on_upgrade({
        let claims = claims.clone();
        move |socket| agent::handle_agent(socket, claims, registry)
      }),
      Some("organization") => ws.on_upgrade({
        let claims = claims.clone();

        move |socket| {
          let presence = registry.subscribe_presence();
          rdp::handle_viewer(socket, claims, registry, presence, certificate)
        }
      }),
      _ => Response::builder()
        .status(401)
        .body("invalid role".into())
        .unwrap(),
    },
    Err(error) => {
      tracing::error!(?error, "WebSocket authentication failed");
      Response::builder()
        .status(401)
        .body("Unauthorized".into())
        .unwrap()
    }
  }
}
