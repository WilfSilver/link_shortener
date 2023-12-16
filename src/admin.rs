//! Handles any interfaces which requires the user to be logged in to access

use diesel_async::AsyncConnection;
use rocket::fairing::AdHoc;
use rocket::response::Redirect;
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use crate::api::API_LOCAL;
use crate::auth::{self, User};
use crate::database::{Db, PrefixLink};
use crate::utils::random_colour;

/// Once a user is logged in, show the admin panel with the prefixes which the
/// user is allowed to use
#[get("/")]
pub async fn index(mut db: Connection<Db>, user: User) -> Template {
    let prefixes: Vec<PrefixLink> = db
        .transaction(|conn| {
            Box::pin(async move {
                Ok::<_, diesel::result::Error>(PrefixLink::get_all(conn, &user.id).await)
            })
        })
        .await
        .unwrap_or_default();

    Template::render(
        "shortener",
        context! {
            api: API_LOCAL,
            colour: random_colour(),
            allow_custom_name: !prefixes.is_empty(),
            prefixes: prefixes,
            name: "Home",
        },
    )
}

/// Redirect to the login page if the user is not logged in (so without the
/// cookie)
#[get("/", rank = 2)]
fn no_auth_index() -> Redirect {
    Redirect::to(uri!(auth::login_page))
}

/// Adds the endpoints for admin interface
pub fn stage(route: String) -> AdHoc {
    AdHoc::on_ignite("Admin Server Initialisation", |rocket| async {
        rocket.mount(route, routes![index, no_auth_index])
    })
}
