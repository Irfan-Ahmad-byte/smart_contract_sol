# Project Setup with SQLx

This guide outlines how to use SQLx for managing database migrations in your Rust projects.

## Installation

To install the SQLx CLI, run the following command:

```sh
cargo install sqlx-cli
```

## Initialization

To initialize SQLx in your project, run:

```
sqlx init
```

## Creating Migrations

To create a new migration file, use:

```
sqlx migrate add <migration_name>
Example: sqlx migrate add create_users_table
```

## Running Migrations

To apply all pending migrations, execute:

```
sqlx migrate run
```

## Reverting Migrations

To revert the last applied migration, use:

```
sqlx migrate revert
```

## Preparing SQLx

Prepare your project to work with SQLx in offline mode by running:

```
cargo sqlx prepare
```

## To verify the correctness of the preparation:

```
cargo sqlx prepare --check
```