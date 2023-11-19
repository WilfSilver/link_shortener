// @generated automatically by Diesel CLI.

diesel::table! {
    urls (name) {
        name -> Varchar,
        url -> Text,
    }
}
