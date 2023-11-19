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
hostname = "http://localhost:8000/"
username = "<username>"
password = "<password_hash>"

[default.databases.diesel_postgres]
url = "<postgresql_url>"
```
