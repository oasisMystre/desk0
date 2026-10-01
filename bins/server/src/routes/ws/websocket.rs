use axum::{
  extract::{Query, State, WebSocketUpgrade},
  http::HeaderMap,
  response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::routes::{
  state::AppState,
  ws::{agent::AgentHandler, context::WsContext, rdp::RdpHandler},
};

#[derive(Deserialize)]
pub struct WebSocketQueryParams {
  pub token: Option<String>,
}

pub async fn ws_route(
  State(AppState {
    ws_context: WsContext {
      hub,
      auth,
      certificate,
    },
    ..
  }): State<AppState>,
  ws: WebSocketUpgrade,
  headers: HeaderMap,
  Query(WebSocketQueryParams { token }): Query<WebSocketQueryParams>,
) -> impl IntoResponse {
  let token = if token.is_some() {
    token
  } else {
    headers
      .get("x-token")
      .and_then(|token| token.to_str().ok())
      .map(|value| value.to_string())
  };

  let unauthorized = Response::builder()
    .status(401)
    .body("Unauthorized".into())
    .unwrap();

  if let Some(token) = token {
    return match auth.authenticate(&token) {
      Ok(claims) if claims.role.as_deref() == Some("agent") => ws.on_upgrade({
        let claims = claims.clone();
        async move |socket| {
          let mut handler = AgentHandler::new(hub);
          if let Err(error) = handler.handle(socket, claims).await {
            tracing::error!(?error, "agent session ended with error");
          }
        }
      }),
      Ok(_) => Response::builder()
        .status(401)
        .body("invalid role".into())
        .unwrap(),
      Err(error) => {
        tracing::error!(?error, "agent WebSocket authentication failed");
        unauthorized
      }
    };
  }

  ws.on_upgrade({
    async move |socket| {
      let mut handler = RdpHandler::new(hub, auth, certificate);
      if let Err(error) = handler.handle(socket).await {
        tracing::error!(?error, "rdp session ended with error");
      }
    }
  })
}
