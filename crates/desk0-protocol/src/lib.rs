pub mod proto {
  tonic::include_proto!("desk0");
}
pub mod bridge;

pub use bridge::*;
pub use proto::{
  desk0_hub_service_client::Desk0HubServiceClient,
  desk0_hub_service_server::{Desk0HubService, Desk0HubServiceServer},
  *,
};
