use utoipa::{
  Modify, OpenApi,
  openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use crate::{
  error::{ErrorResponse, SharedErrorCode, SharedErrorResponse},
  routes::{
    agent::{
      dto::{
        Agents, BuildAgentPayload, BuildAgentRequest, BuildAgentResponse, BuildAgentStatusRequest,
        BuildAgentStatusResponse,
      },
      models::{Agent, AgentAsset, AgentOs},
    },
    applications::{
      dto::AddDeviceApplicationRequest,
      models::{
        Application, DeviceApplication, ResolvedApplication, ResolvedDeviceApplication, Scripts,
      },
    },
    attachments::{
      dto::{CreateUploadRequest, CreateUploadResponse, UpdateAttachmentRequest},
      models::Attachment,
    },
    auth::{
      dto::{
        EmailSignUpRequest, EmailSignUpResponse, RefreshTokenRequest, RefreshTokenResponse,
        RevokeSessionResponse, SessionResponse, VerifyOtpRequest, VerifyOtpResponse,
      },
      models::{Session, User},
    },
    devices::{
      dto::{
        ChallengeResponse, CreateConnectionTokenRequest,
        CreateConnectionTokenResponse as DeviceConnectionTokenResponse, CreateDevice,
        PartialDevice,
      },
      models::Device,
    },
    organizations::{
      dto::{
        CreateConnectionTokenResponse as OrgConnectionTokenResponse, CreateOrganizationRequest,
      },
      models::Organization,
    },
  },
};

struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    if let Some(components) = openapi.components.as_mut() {
      components.add_security_scheme(
        "bearer_auth",
        SecurityScheme::Http(
          HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .build(),
        ),
      );
    }
  }
}

#[derive(OpenApi)]
#[openapi(
  paths(
    crate::routes::auth::handlers::email_signup,
    crate::routes::auth::handlers::verify_otp,
    crate::routes::auth::handlers::refresh_token,
    crate::routes::auth::handlers::get_sessions,
    crate::routes::auth::handlers::revoke_session,
    crate::routes::auth::handlers::logout,
    crate::routes::users::handlers::delete_user,
    crate::routes::organizations::handlers::get_organizations,
    crate::routes::organizations::handlers::create_organization,
    crate::routes::organizations::handlers::create_organization_connection_token,
    crate::routes::devices::handlers::create_device,
    crate::routes::devices::handlers::update_device,
    crate::routes::devices::handlers::create_challenge,
    crate::routes::devices::handlers::create_connection_token,
    crate::routes::devices::handlers::get_devices,
    crate::routes::devices::handlers::delete_device,
    crate::routes::agent::handlers::build_agent,
    crate::routes::agent::handlers::get_build_agent_status,
    crate::routes::agent::handlers::get_agents,
    crate::routes::attachments::handlers::get_attachments,
    crate::routes::attachments::handlers::create_attachment,
    crate::routes::attachments::handlers::update_attachment,
    crate::routes::attachments::handlers::delete_attachment,
    crate::routes::applications::handlers::create_application,
    crate::routes::applications::handlers::get_applications,
    crate::routes::applications::handlers::get_application,
    crate::routes::applications::handlers::delete_application,
    crate::routes::applications::handlers::get_device_applications,
    crate::routes::applications::handlers::add_device_application,
    crate::routes::applications::handlers::remove_device_application,
  ),
  components(
    schemas(
      ErrorResponse,
      SharedErrorResponse,
      SharedErrorCode,
      EmailSignUpRequest,
      EmailSignUpResponse,
      VerifyOtpRequest,
      VerifyOtpResponse,
      RefreshTokenRequest,
      RefreshTokenResponse,
      SessionResponse,
      RevokeSessionResponse,
      User,
      Session,
      CreateOrganizationRequest,
      OrgConnectionTokenResponse,
      Organization,
      CreateDevice,
      PartialDevice,
      DeviceConnectionTokenResponse,
      ChallengeResponse,
      CreateConnectionTokenRequest,
      Device,
      BuildAgentRequest,
      BuildAgentResponse,
      BuildAgentPayload,
      BuildAgentStatusRequest,
      BuildAgentStatusResponse,
      Agents,
      Agent,
      AgentAsset,
      AgentOs,
      CreateUploadRequest,
      CreateUploadResponse,
      UpdateAttachmentRequest,
      Attachment,
      AddDeviceApplicationRequest,
      Application,
      DeviceApplication,
      ResolvedApplication,
      ResolvedDeviceApplication,
      Scripts,
    )
  ),
  modifiers(&SecurityAddon),
  tags(
    (name = "auth", description = "Authentication and session management"),
    (name = "users", description = "User profile operations"),
    (name = "organizations", description = "Organization management"),
    (name = "devices", description = "Device registration, challenge, and management"),
    (name = "agents", description = "Agent build and management"),
    (name = "attachments", description = "File attachments and uploads"),
    (name = "applications", description = "Applications catalog and deployment")
  ),
  info(
    title = "Desk0 Server API",
    version = "0.1.0",
    description = "Desk0 Relay Server & Device Broker OpenAPI specification",
    license(
        name = "MIT",
        identifier = "MIT"
    )
  ),

)]
pub struct ApiDoc;
