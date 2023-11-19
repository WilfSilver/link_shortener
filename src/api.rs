use password_auth::verify_password;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use rocket_db_pools::diesel::prelude::*;
use validator::{Validate, ValidationError};

use crate::admin::User;
use crate::config::AppConfig;
use crate::database::{self, Db, Result};
use crate::schema;

use rocket_db_pools::Connection;

pub static API_LOCAL: &str = "/api/v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PostStatus {
    success: bool,
    message: Option<String>,
}

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

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AddPostResponse {
    success: bool,
    form_errors: Vec<FormErrorPair>,
    error: Option<String>,
    url: Option<String>,
    allow_force: bool,
}

impl AddPostResponse {
    fn ok(url: String) -> Self {
        AddPostResponse {
            success: true,
            form_errors: Vec::new(),
            error: None,
            url: Some(url),
            allow_force: false,
        }
    }

    fn dialog(message: &str, form_errors: Option<Vec<FormErrorPair>>) -> Self {
        AddPostResponse {
            success: false,
            form_errors: form_errors.unwrap_or_default(),
            error: Some(message.to_string()),
            url: None,
            allow_force: true,
        }
    }

    fn error(message: &str, form_errors: Option<Vec<FormErrorPair>>) -> Self {
        AddPostResponse::error_str(message.to_string(), form_errors)
    }

    fn error_str(message: String, form_errors: Option<Vec<FormErrorPair>>) -> Self {
        AddPostResponse {
            success: false,
            form_errors: form_errors.unwrap_or_default(),
            error: Some(message),
            url: None,
            allow_force: false,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize)]
struct AddData {
    #[validate(length(min = 1), custom = "validate_url_name")]
    name: String,
    #[validate(url)]
    url: String,
    force: Option<bool>,
}

fn validate_url_name(name: &str) -> Result<(), ValidationError> {
    let forbidden_names = ["api", "admin", "js", "css"];

    if forbidden_names.into_iter().any(|x| name.eq(x)) {
        return Err(ValidationError::new("Forbidden name"));
    }

    let valid_name = name
        .chars()
        .all(|x| char::is_alphanumeric(x) || x == '/' || x == '-' || x == '_');
    if !valid_name {
        return Err(ValidationError::new("Invalid characters in name!"));
    }

    Ok(())
}

enum AddResultError {
    ElementExists,
    Error(diesel::result::Error),
}

impl From<diesel::result::Error> for AddResultError {
    fn from(value: diesel::result::Error) -> Self {
        AddResultError::Error(value)
    }
}

#[post("/add", data = "<info>")]
async fn add<'r>(
    config: &State<AppConfig>,
    mut db: Connection<Db>,
    _user: User,
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

    let final_url = config.hostname.clone() + &info.name;

    let res = db
        .transaction(|mut conn| {
            Box::pin(async move {
                let res: Result<database::Url, _> = schema::urls::table
                    .filter(schema::urls::name.eq(&info.name))
                    .first(&mut conn)
                    .await;

                let force = info.force.unwrap_or(false);
                let update = force && res.is_ok();
                if !force && res.is_ok() {
                    return Err::<_, AddResultError>(AddResultError::ElementExists);
                }

                if update {
                    diesel::update(schema::urls::table)
                        .filter(schema::urls::name.eq(&info.name))
                        .set(schema::urls::url.eq(&info.url))
                        .execute(&mut conn)
                        .await?;
                } else {
                    diesel::insert_into(schema::urls::table)
                        .values(database::Url {
                            name: info.name.clone(),
                            url: info.url.clone(),
                        })
                        .execute(&mut conn)
                        .await?;
                }

                Ok::<_, AddResultError>(())
            })
        })
        .await;

    match res {
        Ok(_) => Json(AddPostResponse::ok(final_url)),
        Err(e) => match e {
            AddResultError::ElementExists => Json(AddPostResponse::dialog(
                "The name already exists. Would you like to override?",
                None,
            )),
            AddResultError::Error(_) => {
                Json(AddPostResponse::error("Could not create the link", None))
            }
        },
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Login<'v> {
    username: &'v str,
    password: &'v str,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LoginResponse {
    success: bool,
    error: Option<String>,
}

#[post("/login", data = "<login>")]
fn login<'r>(
    config: &State<AppConfig>,
    jar: &CookieJar<'r>,
    login: Json<Login<'r>>,
) -> Json<LoginResponse> {
    if config.username == login.username
        && verify_password(login.password, &config.password).is_ok()
    {
        jar.add_private(("user_id", "1"));
        Json(LoginResponse {
            success: true,
            error: None,
        })
    } else {
        Json(LoginResponse {
            success: false,
            error: Some("Invalid login".to_string()),
        })
    }
}

#[post("/logout")]
fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private("user_id");
    Redirect::to(uri!(crate::admin::login_page))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![add, login, logout]
}
