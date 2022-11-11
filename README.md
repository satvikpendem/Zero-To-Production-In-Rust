# Installation

## Env

Turn env sample file into env

```sh
cp .env.sample .env
```

Don't delete .env.sample because it will pollute the git commit history for next time

## Scripts

Make all scripts executable.

```sh
chmod 777 ./scripts/*
```

## Database

### Create a database user

```sql
create user newsletter with password 'postgres';
alter user newsletter with superuser; -- add superuser role for sqlx-cli to work
create database newsletter; -- optional as `sqlx database create` will create the database if it doesn't exist
```

### Run migrations

```sh
cargo binstall sqlx-cli # or `cargo install sqlx-cli` if you don't have `cargo-binstall` installed
sqlx database create 
sqlx migrate run
```
