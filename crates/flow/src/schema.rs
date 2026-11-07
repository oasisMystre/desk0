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
        expiresAt -> Timestamptz,
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

diesel::table! {
    applications (id) {
        id -> Uuid,
        name -> Text,
        os -> crate::sql_types::Os,
        image -> Nullable<Uuid>,
        asset -> Uuid,
        organization -> Uuid,
        scripts -> Jsonb,
        createdAt -> Timestamptz
    }
}

diesel::table! {
    device_applications (id) {
        id -> Uuid,
        device -> Uuid,
        organization -> Uuid,
        application -> Uuid,
        createdAt -> Timestamptz
    }
}

diesel::allow_tables_to_appear_in_same_query!(organizations, users);
diesel::allow_tables_to_appear_in_same_query!(agents, agent_assets);
diesel::allow_tables_to_appear_in_same_query!(devices, applications);
diesel::allow_tables_to_appear_in_same_query!(devices, attachments);
diesel::allow_tables_to_appear_in_same_query!(devices, device_applications);
diesel::allow_tables_to_appear_in_same_query!(applications, attachments);
diesel::allow_tables_to_appear_in_same_query!(applications, organizations);
diesel::allow_tables_to_appear_in_same_query!(device_applications, attachments);
diesel::allow_tables_to_appear_in_same_query!(device_applications, applications);

pub mod sql_types {
  #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
  #[diesel(postgres_type(name = "os"))]
  pub struct Os;
}
