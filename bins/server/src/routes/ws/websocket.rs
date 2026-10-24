use axum::{
  extract::{Query, State, WebSocketUpgrade},
  http::HeaderMap,
  response::IntoResponse,
};
use serde::Deserialize;

use crate::{
  error::SharedError,
  routes::ws::{agent::AgentHandler, context::WsContext, rdp::RdpHandler},
  states::AppState,
};

#[derive(Deserialize)]
pub struct WebSocketQueryParams {
  pub token: Option<String>,
}

pub async fn ws_route(
  State(AppState {
    context,
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

  let unauthorized = SharedError::WsAuthFailed;

  if let Some(token) = token {
    return match auth.authenticate(&token) {
      Ok(claims) if claims.role.as_deref() == Some("agent") => {
        if let Err(error) =
          crate::states::consume_connection_token(&context, &token, claims.jti.as_deref()).await
        {
          tracing::warn!(?error, "failed to consume agent connection token");
          return unauthorized.into_response();
        }

        ws.on_upgrade({
          let claims = claims.clone();
          async move |socket| {
            let mut handler = AgentHandler::new(hub);
            if let Err(error) = handler.handle(socket, claims).await {
              tracing::error!(?error, "agent session ended with error");
            }
          }
        })
      }
      Ok(_) => SharedError::WsInvalidRole.into_response(),
      Err(error) => {
        tracing::error!(?error, "agent WebSocket authentication failed");
        unauthorized.into_response()
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
