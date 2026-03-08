# Instructions

I want you to make a rust crate in this directory called `sqlx-testcontainers`.
The purpose of this repo is to provide an integration between `sqlx` and `testcontainers`.

## Provided Interface

Provide a macro called `#{sqlx_testcontainers::test]`.
This macro should transform a function of type `async fn test_name_here(sqlx::postgres::PgConnection conn)` into a proper test.
In order to do this, the macro should:

1. Spin up a postgres test container (using `testcontainers-modules`)
2. Run sqlx migrations in this container
3. Acquire a database connection for this container
4. Pass it to the provided function the macro was called on


## Configuraiton

We want to be able to configure the macro to add some configurability.
Specifically, users should be able to pass an argument to the macro to specify the *tag* of the postgres test container.
This should use the `with_tag` method already provided in `testcontainers_modules`.
