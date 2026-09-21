use axum::{
  extract::{Query, State, WebSocketUpgrade},
  response::{IntoResponse, Response},
};
use axum_extra::{
  TypedHeader,
  headers::{Authorization, authorization::Bearer},
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
  authorization: Option<TypedHeader<Authorization<Bearer>>>,

  Query(WebSocketQueryParams { token }): Query<WebSocketQueryParams>,
) -> impl IntoResponse {
  let Some(token) = token.or_else(|| authorization.map(|value| value.0.token().to_string())) else {
    return Response::builder()
      .status(401)
      .body("Unauthorized".into())
      .unwrap();
  };

  let token = token.to_string();
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
