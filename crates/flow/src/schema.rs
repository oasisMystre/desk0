
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

diesel::allow_tables_to_appear_in_same_query!(organizations, users);
