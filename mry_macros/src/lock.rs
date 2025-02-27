use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, ItemFn};

pub struct LockPaths(Vec<syn::Path>);

impl syn::parse::Parse for LockPaths {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input
                .parse_terminated(syn::Path::parse, syn::Token![,])?
                .into_iter()
                .collect(),
        ))
    }
}

pub(crate) fn transform(args: LockPaths, mut input: ItemFn) -> TokenStream {
    let args = args.0.into_iter().map(|arg| {
        let name = arg.to_token_stream().to_string().replace(' ', "");
        quote![(std::any::Any::type_id(&#arg), #name.to_string())]
    });
    let block = input.block.clone();
    input.block.stmts.clear();
    let mutexes = quote![mry::__mutexes(vec![#(#args,)*])];
    input.block.stmts.insert(
        0,
        syn::Stmt::Expr(
            if input.sig.asyncness.is_some() {
                parse_quote! {
                    mry::__async_lock_and_run(#mutexes, move || Box::pin(async #block)).await
                }
            } else {
                parse_quote! {
                    mry::__lock_and_run(#mutexes, move || #block)
                }
            },
            None,
        ),
    );
    input.into_token_stream()
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use syn::{parse2, parse_str};

    use super::*;

    #[test]
    fn lock() {
        let args = LockPaths(vec![parse_str("a::a").unwrap(), parse_str("b::b").unwrap()]);
        let input: ItemFn = parse2(quote! {
            #[test]
            fn test_meow() {
                assert!(true);
            }
        })
        .unwrap();

        assert_eq!(
            transform(args, input).to_string(),
            quote! {
                #[test]
                fn test_meow() {
                    mry::__lock_and_run(mry::__mutexes(vec![
                        (std::any::Any::type_id(&a :: a), "a::a".to_string()),
                        (std::any::Any::type_id(&b :: b), "b::b".to_string()),
                    ]), move | | {
                        assert!(true);
                    })
                }
            }
            .to_string()
        );
    }
}
