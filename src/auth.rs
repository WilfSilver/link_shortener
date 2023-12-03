use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl,
    TokenResponse,
};
use rocket::{
    fairing::AdHoc,
    http::{CookieJar, Status},
};
use rocket::{http::Cookie, outcome::IntoOutcome};
use rocket::{
    http::SameSite,
    request::{self, FromRequest, Request},
};
use rocket::{response::Redirect, serde::json, State};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

pub const USER_COOKIE: &str = "user";
pub const VALIDATOR_COOKIE: &str = "validator";

#[derive(Debug, Serialize, Deserialize)]
pub struct OidcValidator {
    pub auth_url: String,
    pub csrf_token: CsrfToken,
    pub nonce: Nonce,
}

// Inspired by https://github.com/csssuf/rocket_oidc
impl OidcValidator {
    pub fn new(client: &CoreClient) -> Self {
        let (auth_url, csrf_token, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .url();

        OidcValidator {
            auth_url: auth_url.to_string(),
            csrf_token,
            nonce,
        }
    }

    pub async fn verify(
        &self,
        client: &CoreClient,
        code: &str,
    ) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let tr = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            // .set_pkce_verifier(self.pkce_verifier)
            .request_async(async_http_client)
            .await?;

        let id_token = match tr.id_token() {
            Some(x) => x,
            None => return Ok(None),
        };

        let claims = id_token.claims(&client.id_token_verifier(), &self.nonce)?;

        Ok(Some(User {
            id: claims.subject().to_string(),
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
        request
            .cookies()
            .get_private(USER_COOKIE)
            .and_then(|cookie| json::from_str::<User>(cookie.value()).ok())
            .or_forward(Status::Unauthorized)
    }
}

pub async fn get_client(config: &AppConfig) -> Result<CoreClient, Box<dyn std::error::Error>> {
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(config.client_url.clone())?,
        async_http_client,
    )
    .await?;

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(config.client_id.to_string()),
        Some(ClientSecret::new(config.client_secret.to_string())),
    )
    .set_redirect_uri(RedirectUrl::new(config.hostname.clone() + "callback")?);

    Ok(client)
}

#[get("/callback?<code>", rank = 2)]
async fn callback<'r>(
    jar: &CookieJar<'r>,
    client: &State<CoreClient>,
    code: &str,
) -> Result<Redirect, String> {
    let val = jar
        .get_private(VALIDATOR_COOKIE)
        .and_then(|cookie| json::from_str::<OidcValidator>(cookie.value()).ok());

    jar.remove_private(VALIDATOR_COOKIE);

    if let Some(validator) = val {
        if let Some(user) = validator
            .verify(client, code)
            .await
            .map_err(|e| e.to_string())?
        {
            jar.add_private(
                Cookie::build((USER_COOKIE, json::to_string(&user).unwrap()))
                    .same_site(SameSite::Lax),
            );
            return Ok(Redirect::to(uri!("/admin")));
        }
    }

    Ok(Redirect::to(uri!("/")))
}

#[get("/callback")]
fn callback_no_auth(_user: User) -> Redirect {
    Redirect::to(uri!("/admin"))
}

#[get("/login")]
fn login(_user: User) -> Redirect {
    Redirect::to(uri!("/admin"))
}

#[get("/login", rank = 2)]
pub fn login_page(jar: &CookieJar, client: &State<CoreClient>) -> Redirect {
    let validator = OidcValidator::new(client);
    jar.add_private(
        Cookie::build((VALIDATOR_COOKIE, json::to_string(&validator).unwrap()))
            .same_site(SameSite::Lax),
    );
    Redirect::to(validator.auth_url)
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Authentication Server Stage", |rocket| async {
        let config: AppConfig = rocket
            .figment()
            .extract()
            .expect("Could not find App Config");
        let client = get_client(&config)
            .await
            .expect("Could not find authentication server");

        rocket
            .manage(client)
            .mount("/", routes![login, login_page, callback, callback_no_auth])
    })
}
