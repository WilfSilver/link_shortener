//! Stores the Database structures and functions which can be used for
//! interacting with the database via diesel

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
    /// Returns whether a URL already exists in the database
    pub async fn exists(conn: &mut Connection<Db>, name: &str) -> bool {
        let res: Result<Url, _> = schema::urls::table
            .filter(schema::urls::name.eq(name))
            .first(conn)
            .await;

        res.is_ok()
    }

    /// Gets the row from the URL
    pub async fn from_url(conn: &mut Connection<Db>, url: &str) -> Option<Url> {
        let res: Result<Url, _> = schema::urls::table
            .filter(schema::urls::url.eq(url))
            .first(conn)
            .await;

        res.ok()
    }
}

#[derive(Deserialize, Insertable, Queryable, Serialize, Selectable)]
#[diesel(table_name = crate::schema::prefixes)]
#[serde(crate = "rocket::serde")]
pub struct PrefixLink {
    pub user_id: String,
    pub prefix: String,
}

impl PrefixLink {
    /// Returns all the prefixes which a given user is allowed to use
    pub async fn get_all(conn: &mut Connection<Db>, user_id: &str) -> Vec<PrefixLink> {
        schema::prefixes::table
            .filter(schema::prefixes::user_id.eq(user_id))
            .get_results(conn)
            .await
            .unwrap_or_default()
    }

    /// Returns if a user is allowed to use a link with a given name
    pub async fn user_can_link(conn: &mut Connection<Db>, user_id: &str, link_name: &str) -> bool {
        let prefixes = PrefixLink::get_all(conn, user_id).await;
        for p in prefixes {
            let length = p.prefix.len();

            if length == 0 || (link_name.len() >= length && p.prefix == link_name[..length]) {
                return true;
            }
        }

        false
    }
}

/// Initialises the database
pub fn stage() -> AdHoc {
    AdHoc::on_ignite("PostgreSQL Stage", |rocket| async {
        rocket.attach(Db::init())
    })
}
