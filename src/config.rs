//! Stores custom configuration information

use figment::{
    providers::{Env, Format, Toml},
    Figment, Profile,
};
use serde::{Deserialize, Serialize};

/// Custom config options used throughout the application
#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub client_url: String,
    pub hostname: String,
}

pub fn get_figment() -> Figment {
    Figment::from(rocket::Config::default())
        .merge(Toml::file("Rocket.toml").nested())
        .merge(Env::prefixed("APP_").global())
        .select(Profile::from_env_or("APP_PROFILE", "default"))
}
