use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, ItemFn, LitStr, Token, Ident};

struct MacroArgs {
    tag: Option<String>,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(MacroArgs { tag: None });
        }

        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            let name: Ident = input.parse()?;
            if name == "tag" {
                input.parse::<Token![=]>()?;
                let tag_lit: LitStr = input.parse()?;
                Ok(MacroArgs { tag: Some(tag_lit.value()) })
            } else {
                Err(syn::Error::new(name.span(), "expected `tag`"))
            }
        } else if lookahead.peek(LitStr) {
            let tag_lit: LitStr = input.parse()?;
            Ok(MacroArgs { tag: Some(tag_lit.value()) })
        } else {
            Err(lookahead.error())
        }
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
            
            ::sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");
            
            let mut conn = pool.acquire().await.expect("Failed to acquire connection").detach();
            
            #input

            #inner_name(conn).await;
        }
    };

    TokenStream::from(expanded)
}
