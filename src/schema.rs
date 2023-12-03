// @generated automatically by Diesel CLI.

diesel::table! {
    prefixes (user_id, prefix) {
        user_id -> Varchar,
        prefix -> Text,
    }
}

diesel::table! {
    urls (name) {
        name -> Varchar,
        url -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    prefixes,
    urls,
);
