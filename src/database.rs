use rocket::fairing::AdHoc;
use rocket::response::Debug;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::diesel::{self, prelude::*, PgPool};
use rocket_db_pools::Database;

pub type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[derive(Database)]
#[database("diesel_postgres")]
pub struct Db(PgPool);

#[derive(Deserialize, Insertable, Queryable, Serialize, Selectable)]
#[diesel(table_name = crate::schema::urls)]
#[serde(crate = "rocket::serde")]
pub struct Url {
    pub name: String,
    pub url: String,
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("PostgreSQL Stage", |rocket| async {
        rocket.attach(Db::init())
    })
}
