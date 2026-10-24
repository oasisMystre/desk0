diesel::table! {
    users (id) {
        id -> Uuid,
        uid -> Nullable<Text>,
        lastLoggedIn -> Timestamptz,
        createdAt -> Timestamptz
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user -> Uuid,
        userAgent -> Nullable<Text>,
        ipAddress -> Nullable<Inet>,
        expiresAt -> Timestamptz,
        createdAt -> Timestamptz,
        lastUsedAt -> Nullable<Timestamptz>,
        revokedAt -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        session -> Uuid,
        token -> Bytea,
        expiredAt -> Timestamptz,
        createdAt -> Timestamptz,
        consumedAt -> Nullable<Timestamptz>,
        revokedAt -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
        name -> Nullable<Text>,
        user -> Uuid,
        createdAt -> Timestamptz
    }
}

diesel::table! {
    devices (id) {
        id -> Uuid,
        pubkey -> Bytea,
        organization -> Uuid,
        os -> Nullable<Text>,
        version -> Nullable<Text>,
        hostname -> Nullable<Text>,
        username -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        lastLoggedIn -> Timestamptz,
        createdAt -> Timestamptz
    }
}

diesel::table! {
    attachments (id) {
        id -> Uuid,
        key -> Text,
        filename -> Text,
        publicUrl -> Text,
        uploaded -> Bool,
        contentType -> Text,
        organization -> Uuid,
        createdAt -> Timestamptz,
        metadata -> Nullable<Jsonb>
    }
}

diesel::table! {
    agents (id) {
        id -> Uuid,
        organization -> Uuid,
        version -> Int4,
        createdAt -> Timestamptz
    }
}

diesel::table! {
    agent_assets (agent, os, arch) {
        agent -> Uuid,
        os -> crate::sql_types::Os,
        arch -> Text,
        version -> Int4,
        attachment -> Uuid
    }
}

diesel::allow_tables_to_appear_in_same_query!(organizations, users);

pub mod sql_types {
  #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
  #[diesel(postgres_type(name = "os"))]
  pub struct Os;
}
