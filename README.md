# Link Shortener

As there are already more than enough link shorteners in the world, I thought
I would make another one.

This requires a connection to a postgres server which is used to store the
links.

## Requirements

- `diesel` for database migrations

## Get Started

```sh
export DATABASE_URL="<postgresql_url"
diesel migration run
cargo run
```

## Configuration

In the `Rocket.toml` file you need the following information

```sh
[default.databases.diesel_postgres]
url = "<postgresql_url>"
```

And then you need to specify the following environmental variables (recommended
to put them in the `.cargo/config.toml` file)

```sh
APP_CLIENT_ID="<client_id>"
APP_CLIENT_SECRET="<client_secret>"
APP_CLIENT_URL="<oidc_server_url>"
APP_HOSTNAME="<hostname>"

APP_SECRET_KEY="<your_secret_key>"
```
