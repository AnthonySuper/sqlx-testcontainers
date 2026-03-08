use sqlx::Row;

#[sqlx_testcontainers::test]
async fn test_basic(mut conn: sqlx::postgres::PgConnection) {
    let row: (i32,) = sqlx::query_as("SELECT 1 + 1")
        .fetch_one(&mut conn)
        .await
        .expect("Failed to execute query");
    assert_eq!(row.0, 2);
}

#[sqlx_testcontainers::test(tag = "14-alpine")]
async fn test_with_tag(mut conn: sqlx::postgres::PgConnection) {
    let row: (String,) = sqlx::query_as("SELECT version()")
        .fetch_one(&mut conn)
        .await
        .expect("Failed to execute query");
    assert!(row.0.contains("PostgreSQL 14"));
}

#[sqlx_testcontainers::test(migrations = "migrations")]
async fn test_with_migrations_path(mut conn: sqlx::postgres::PgConnection) {
    let row: (i32,) = sqlx::query_as("SELECT count(*) FROM test")
        .fetch_one(&mut conn)
        .await
        .expect("Failed to execute query");
    assert_eq!(row.0, 0);
}
