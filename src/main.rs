#[macro_use]
extern crate rocket;

use api::API_LOCAL;
use database::Url;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::http::Status;
use rocket::response::Redirect;
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::Connection;
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;

mod admin;
mod api;
mod config;
mod database;
mod schema;
mod utils;

#[cfg(test)]
mod tests;

use crate::config::AppConfig;
use crate::database::{Db, Result};
use crate::utils::random_colour;

#[get("/")]
pub fn index() -> Redirect {
    Redirect::to(uri!("/admin"))
}

#[get("/<link>", rank = 100)]
async fn redirect(mut db: Connection<Db>, link: &str) -> Result<Redirect, Status> {
    let res: Result<Url, _> = schema::urls::table
        .filter(schema::urls::name.eq(link))
        .first(&mut db)
        .await;

    match res {
        Ok(url) => Ok(Redirect::to(url.url)),
        Err(_) => Err(Status::NotFound),
    }
}

#[catch(500)]
fn internal_error() -> Template {
    Template::render(
        "error/500",
        context! {
            colour: random_colour(),
            name: "Oops",
        },
    )
}

#[catch(404)]
fn not_found() -> Template {
    Template::render(
        "error/404",
        context! {
            colour: random_colour(),
            name: "Lost and Found",
        },
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .attach(AdHoc::config::<AppConfig>())
        .attach(database::stage())
        .mount("/admin", admin::routes())
        .mount(API_LOCAL, api::routes())
        .mount("/", routes![index, redirect])
        .mount("/", FileServer::from(relative!("static")))
        .register("/", catchers![not_found, internal_error])
}
