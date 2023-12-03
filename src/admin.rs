use diesel_async::AsyncConnection;
use rocket::response::Redirect;
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use crate::api::API_LOCAL;
use crate::auth::{self, User};
use crate::database::{Db, PrefixLink};
use crate::utils::random_colour;

#[get("/")]
pub async fn index(mut db: Connection<Db>, user: User) -> Template {
    println!("User: {}", user.id);
    let prefixes: Vec<PrefixLink> = db
        .transaction(|conn| {
            Box::pin(async move {
                Ok::<_, diesel::result::Error>(PrefixLink::get_all(conn, &user.id).await)
            })
        })
        .await
        .unwrap_or_default();

    println!("Prefixes: {}", prefixes.len());

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

#[get("/", rank = 2)]
fn no_auth_index() -> Redirect {
    Redirect::to(uri!(auth::login_page))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index, no_auth_index]
}
