# Link Shortener

As there are already more than enough link shorteners in the world, I thought
I would make another one.

This requires a connection to a postgres server which is used to store the
links.

## Requirements

- `diesel` for database migrations

## Get Started

```sh
export DATABASE_URL="<postgresql_url>"
diesel migration run
cargo run
```

## Configuration

We recommend that you set these parameters in the environmental variables for
security, however they can also be set in the `Rocket.toml` file by following
their documentation.

If you want this to work with `cargo run`, you can use the `.cargo/config.toml`
and put the variables under `[env]`, or you can have the following `.env` file:

```sh
APP_CLIENT_ID="<client_id>"
APP_CLIENT_SECRET="<client_secret>"
APP_CLIENT_URL="<oidc_server_url>"
APP_HOSTNAME="<hostname>"

APP_DATABASES="{diesel_postgres={url=\"<database_url>\",idle_timeout=120}}"
APP_SECRET_KEY="<your_secret_key>"
```
