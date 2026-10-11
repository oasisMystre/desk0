diesel::table! {
    users (id) {
        id -> Uuid,
        uid -> Nullable<Text>,
        lastLoggedIn -> Timestamptz,
        createdAt -> Timestamptz
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
        os -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        hostname -> Nullable<Text>,
        username -> Nullable<Text>,
        version -> Nullable<Text>,
        organization -> Uuid,
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
