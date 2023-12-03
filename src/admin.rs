use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};

use crate::auth::User;
use crate::utils::random_colour;

use crate::api::API_LOCAL;
use crate::auth;

#[get("/")]
pub fn index(_user: User) -> Template {
    Template::render(
        "shortener",
        context! { api: API_LOCAL, colour: random_colour(), name: "Home" },
    )
}

#[get("/", rank = 2)]
fn no_auth_index() -> Redirect {
    Redirect::to(uri!(auth::login_page))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index, no_auth_index]
}
