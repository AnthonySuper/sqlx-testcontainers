# sqlx-testcontainers

`sqlx-testcontainers` is a Rust library that simplifies integration testing with [sqlx](https://github.com/launchbadge/sqlx) and [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs). It provides a procedural macro `#[sqlx_testcontainers::test]` that automatically:

1.  Starts a temporary PostgreSQL container using Testcontainers.
2.  Connects to the container and runs your database migrations.
3.  Provides an active `sqlx::PgConnection` directly to your test function.
4.  Cleans up the container after the test finishes.

## Features

- **Zero-Config Database Tests**: No need to manually manage database instances or connection strings for tests.
- **Automatic Migrations**: Uses `sqlx::migrate!()` to ensure your test database schema is up-to-date.
- **Isolated Environments**: Every test gets its own fresh container, ensuring no cross-test interference.
- **Customizable Images**: Easily specify PostgreSQL versions/tags.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sqlx-testcontainers = { git = "https://github.com/your-username/sqlx-testcontainers" }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "migrate"] }
tokio = { version = "1", features = ["full"] }
```

## Usage

Simply replace `#[tokio::test]` with `#[sqlx_testcontainers::test]` and add a `PgConnection` argument to your test function.

```rust
use sqlx::Row;

#[sqlx_testcontainers::test]
async fn test_basic_query(mut conn: sqlx::postgres::PgConnection) {
    let row: (i32,) = sqlx::query_as("SELECT 1 + 1")
        .fetch_one(&mut conn)
        .await
        .expect("Failed to execute query");
        
    assert_eq!(row.0, 2);
}

#[sqlx_testcontainers::test(tag = "15-alpine")]
async fn test_with_specific_version(mut conn: sqlx::postgres::PgConnection) {
    let row: (String,) = sqlx::query_as("SELECT version()")
        .fetch_one(&mut conn)
        .await
        .expect("Failed to execute query");
        
    assert!(row.0.contains("PostgreSQL 15"));
}

#[sqlx_testcontainers::test(migrations = "custom_migrations")]
async fn test_with_custom_migrations(mut conn: sqlx::postgres::PgConnection) {
    // This will use the migrations folder at "custom_migrations"
}
```

## Requirements

- **Docker**: Since this uses Testcontainers, you must have Docker (or a compatible container runtime) installed and running in your environment.
- **SQLx Migrations**: By default, it looks for a `migrations/` folder in your project root to apply to the test database.

## License

MIT or Apache-2.0.
