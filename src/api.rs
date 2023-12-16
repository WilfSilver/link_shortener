//! All endpoints and structures used and returned by the API (which requires
//! authentication to access)

use rand::distributions::Alphanumeric;
use rand::Rng;
use rocket::fairing::AdHoc;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use rocket_db_pools::diesel::prelude::*;
use rocket_db_pools::Connection;
use validator::{Validate, ValidationError};

use crate::auth::{User, USER_COOKIE};
use crate::config::AppConfig;
use crate::database::{self, Db, PrefixLink, Result, Url};
use crate::schema;

pub static API_LOCAL: &str = "/api/v1";

/// This stores the name of the value which is invalid and the description of
/// the error so we can pass it back to the frontend to give more interactive
/// errors.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct FormErrorPair {
    name: String,
    description: String,
}

impl FormErrorPair {
    fn from_validation(name: &str, errors: &[ValidationError]) -> Self {
        FormErrorPair {
            name: name.to_string(),
            description: errors
                .iter()
                .filter_map(|err| err.message.clone().map(|m| m.to_string()))
                .collect::<Vec<String>>()
                .join(", "),
        }
    }
}

/// Type which is returned from the "/add" endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
struct AddPostResponse {
    success: bool,
    form_errors: Vec<FormErrorPair>,
    error: Option<String>,
    url: Option<String>,
    allow_force: bool,
}

impl AddPostResponse {
    /// Returns an okay error with the shortened URL
    fn ok(url: String) -> Self {
        AddPostResponse {
            success: true,
            form_errors: Vec::new(),
            error: None,
            url: Some(url),
            allow_force: false,
        }
    }

    /// Asks the frontend to prompt the user with a warning page, (meaning if
    /// they run with force the request would succeed)
    fn dialog(message: &str, form_errors: Option<Vec<FormErrorPair>>) -> Self {
        AddPostResponse {
            success: false,
            form_errors: form_errors.unwrap_or_default(),
            error: Some(message.to_string()),
            url: None,
            allow_force: true,
        }
    }

    /// Returns a error response
    fn error(message: &str, form_errors: Option<Vec<FormErrorPair>>) -> Self {
        AddPostResponse {
            success: false,
            form_errors: form_errors.unwrap_or_default(),
            error: Some(message.to_string()),
            url: None,
            allow_force: false,
        }
    }
}

/// Data which needs to be given when requesting "/add"
#[derive(Debug, Validate, Deserialize, Serialize)]
struct AddData {
    #[validate(length(min = 1), custom = "validate_url_name")]
    name: Option<String>,
    #[validate(url)]
    url: String,
    force: Option<bool>,
}

/// Validates a valid shorted URL name, making sure it doesn't have any
/// invalid characters.
fn validate_url_name(name: &str) -> Result<(), ValidationError> {
    let forbidden_names = ["api", "admin", "js", "css", "login", "callback"];

    if forbidden_names.into_iter().any(|x| name.eq(x)) {
        return Err(ValidationError::new("Forbidden name"));
    }

    let valid_name = name
        .chars()
        .all(|x| char::is_alphanumeric(x) || x == '-' || x == '_');
    if !valid_name {
        return Err(ValidationError::new("Invalid characters in name!"));
    }

    Ok(())
}

/// Potential errors which can be returned by the add function
enum AddResultError {
    Error(diesel::result::Error),
    FailedGen,
    NameExists,
    UrlExists(String),
    UnauthorisedLink,
}

impl From<diesel::result::Error> for AddResultError {
    fn from(value: diesel::result::Error) -> Self {
        AddResultError::Error(value)
    }
}

/// Generates a random 3 letter name for the shorted URL when one is not given
async fn gen_random_name(conn: &mut Connection<Db>) -> Result<String, AddResultError> {
    // Try 5 times to generate a name before giving up
    for _ in 0..5 {
        let name: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(3)
            .map(char::from)
            .collect();

        if !Url::exists(conn, &name).await {
            return Ok(name);
        }
    }

    Err(AddResultError::FailedGen)
}

/// Returns whether a name should be updated or inserted, or if it exists
/// without force being used, it will return an error which can be passed back
/// to the user
async fn should_update(
    conn: &mut Connection<Db>,
    name: &str,
    url: &str,
    force: bool,
) -> Result<bool, AddResultError> {
    let other_link = Url::from_url(conn, url).await;
    let name_exists: bool = Url::exists(conn, name).await;

    if force {
        Ok(name_exists | other_link.is_some())
    } else if name_exists {
        return Err(AddResultError::NameExists);
    } else if let Some(link) = other_link {
        return Err(AddResultError::UrlExists(link.name));
    } else {
        Ok(false)
    }
}

/// Endpoint for adding a shortened URL
#[post("/add", data = "<info>")]
async fn add<'r>(
    config: &State<AppConfig>,
    mut db: Connection<Db>,
    user: User,
    info: Json<AddData>,
) -> Json<AddPostResponse> {
    if let Err(e) = info.validate() {
        let errors = e
            .field_errors()
            .iter()
            .map(|(name, errors)| FormErrorPair::from_validation(name, errors))
            .collect();

        return Json(AddPostResponse::error("Invalid request", Some(errors)));
    }

    let res = db
        .transaction(|mut conn| {
            Box::pin(async move {
                let (name, update) = match &info.name {
                    Some(name) => {
                        // Check if the user has permission to create a link with
                        // this name
                        if !PrefixLink::user_can_link(conn, &user.id, name).await {
                            return Err(AddResultError::UnauthorisedLink);
                        }

                        let up = should_update(conn, name, &info.url, info.force.unwrap_or(false))
                            .await?;
                        (name.clone(), up)
                    }
                    None => {
                        // If it already exists we just want to return that
                        if let Some(link) = Url::from_url(conn, &info.url).await {
                            return Ok(link.name);
                        }

                        (gen_random_name(conn).await?, false)
                    }
                };

                if update {
                    diesel::update(schema::urls::table)
                        .filter(schema::urls::name.eq(&name))
                        .set(schema::urls::url.eq(&info.url))
                        .execute(&mut conn)
                        .await?;
                } else {
                    diesel::insert_into(schema::urls::table)
                        .values(database::Url {
                            name: name.clone(),
                            url: info.url.clone(),
                        })
                        .execute(&mut conn)
                        .await?;
                }

                Ok::<_, AddResultError>(name)
            })
        })
        .await;

    match res {
        Ok(name) => Json(AddPostResponse::ok(config.hostname.clone() + &name)),
        Err(e) => match e {
            AddResultError::UnauthorisedLink => Json(AddPostResponse::error(
                "You do not have permission to create this link",
                None,
            )),
            AddResultError::NameExists => Json(AddPostResponse::dialog(
                "The name already exists. Would you like to override?",
                None,
            )),
            AddResultError::UrlExists(name) => Json(AddPostResponse::dialog(
                &format!("This already has a link with name '{}'. Are you sure you want to create a new link?", name),
                None,
            )),
            AddResultError::Error(_) | AddResultError::FailedGen => {
                Json(AddPostResponse::error("Could not create the link", None))
            }
        },
    }
}

/// Logs the user out
#[post("/logout")]
fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private(USER_COOKIE);
    Redirect::to(uri!(crate::auth::login_page))
}

/// Initialises the API at a given route
pub fn stage(route: String) -> AdHoc {
    AdHoc::on_ignite("API Server Initialisation", |rocket| async {
        rocket.mount(route, routes![add, logout])
    })
}
