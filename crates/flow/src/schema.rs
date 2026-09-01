diesel::table! {

    organizations (id) {
        id -> Uuid,
        uid -> Nullable<Text>,
        name -> Nullable<Text>,
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
