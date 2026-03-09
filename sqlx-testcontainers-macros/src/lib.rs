use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, ItemFn, LitStr, Token,
};

struct MacroArgs {
    tag: Option<String>,
    migrations: Option<String>,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut tag = None;
        let mut migrations = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            match ident.to_string().as_str() {
                "tag" => tag = Some(value.value()),
                "migrations" => migrations = Some(value.value()),
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "expected `tag` or `migrations`",
                    ))
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(MacroArgs { tag, migrations })
    }
}

#[proc_macro_attribute]
pub fn test(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let mut input = parse_macro_input!(input as ItemFn);

    let test_name = input.sig.ident.clone();
    let inner_name = quote::format_ident!("__inner_{}", test_name);

    // Rename the original function to __inner_...
    input.sig.ident = inner_name.clone();

    let tag_expr = if let Some(t) = args.tag {
        quote! { Some(#t.to_string()) }
    } else {
        quote! { None }
    };

    let migrate_expr = if let Some(m) = args.migrations {
        quote! { ::sqlx::migrate!(#m).run(&pool).await.expect("Failed to run migrations"); }
    } else {
        quote! { ::sqlx::migrate!().run(&pool).await.expect("Failed to run migrations"); }
    };

    let expanded = quote! {
        #[::tokio::test]
        async fn #test_name() {
            use ::testcontainers_modules::testcontainers::ImageExt;
            use ::testcontainers_modules::testcontainers::runners::AsyncRunner;
            use ::testcontainers_modules::postgres::Postgres;

            let mut image = Postgres::default();
            let tag: Option<String> = #tag_expr;
            let container = if let Some(tag) = tag {
                image.with_tag(tag).start().await.expect("Failed to start postgres container")
            } else {
                image.start().await.expect("Failed to start postgres container")
            };

            let host = container.get_host().await.expect("Failed to get host");
            let host_port = container.get_host_port_ipv4(5432).await.expect("Failed to get port");

            let conn_str = format!("postgres://postgres:postgres@{}:{}/postgres", host, host_port);

            let pool = ::sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect(&conn_str)
                .await
                .expect("Failed to connect to postgres");

            #migrate_expr

            let mut conn = pool.acquire().await.expect("Failed to acquire connection").detach();

            #input

            #inner_name(conn).await;
        }
    };

    TokenStream::from(expanded)
}
