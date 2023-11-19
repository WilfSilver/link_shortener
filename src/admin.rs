use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};

use crate::utils::random_colour;

#[derive(Debug)]
pub struct User(usize);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
        request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(User)
            .or_forward(Status::Unauthorized)
    }
}

#[macro_export]
macro_rules! admin_uri {
    ($($t:tt)*) => (rocket::uri!("/admin", $crate::admin:: $($t)*))
}

pub use admin_uri as uri;

use crate::api::API_LOCAL;

#[get("/")]
pub fn index(_user: User) -> Template {
    Template::render(
        "shortener",
        context! { api: API_LOCAL, colour: random_colour(), name: "Home" },
    )
}

#[get("/", rank = 2)]
fn no_auth_index() -> Redirect {
    Redirect::to(uri!(login_page))
}

#[get("/login")]
fn login(_user: User) -> Redirect {
    Redirect::to(uri!(index))
}

#[get("/login", rank = 2)]
fn login_page() -> Template {
    Template::render(
        "login",
        context! {
            api: API_LOCAL,
            colour: random_colour(),
            homepage: "/admin",
            name: "Login",
        },
    )
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index, no_auth_index, login, login_page]
}
