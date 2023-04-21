# Zero2prod Rust

![GitHub issues](https://img.shields.io/github/issues/AdeThorMiwa/zero2prod.svg)
[![Rust](https://github.com/AdeThorMiwa/zero2prod/actions/workflows/ci.yml/badge.svg)](https://github.com/AdeThorMiwa/zero2prod/actions/workflows/ci.yml)

## Install PSQL

```bash
brew install libpq

echo '
# For PSQL and other LIBPQ binaries
export PATH="/usr/local/opt/libpq/bin:$PATH"' >> ~/.zshrc
```

## Instal SQLX CLI for DB Migrations

```bash
cargo install --version="~0.6" sqlx-cli --no-default-features --features rustls,postgres

sqlx --help
```

## Create DB

```bash
./scripts/init_db.sh
```

## Create DB Migrations Folder

```bash
export DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/newsletter
sqlx migrate add create_subscriptions_table
```

## Run Migration

```bash
SKIP_DOCKER=true ./scripts/init_db.sh
```

## Check Performed Migrations

```bash
psql "postgresql://postgres:password@localhost/newsletter" -c "SELECT * FROM _sqlx_migrations"
```
