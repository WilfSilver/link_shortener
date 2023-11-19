use rocket::fairing::AdHoc;
use rocket::response::Debug;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::diesel::{self, prelude::*, PgPool};
use rocket_db_pools::{Connection, Database};

use crate::schema;

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

impl Url {
    pub async fn exists(conn: &mut Connection<Db>, name: &str) -> bool {
        let res: Result<Url, _> = schema::urls::table
            .filter(schema::urls::name.eq(name))
            .first(conn)
            .await;

        res.is_ok()
    }

    pub async fn from_url(conn: &mut Connection<Db>, url: &str) -> Option<Url> {
        let res: Result<Url, _> = schema::urls::table
            .filter(schema::urls::url.eq(url))
            .first(conn)
            .await;

        res.ok()
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("PostgreSQL Stage", |rocket| async {
        rocket.attach(Db::init())
    })
}
