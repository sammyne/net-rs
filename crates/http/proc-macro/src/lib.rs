use core::panic;

use proc_macro2::{Span, TokenStream};

use quote::quote;
use syn::{FnArg, Ident, ItemFn, Lifetime, Pat, Type};

#[proc_macro_attribute]
pub fn handler_func(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    build_handler_func(TokenStream::from(attr), TokenStream::from(item)).into()
}

fn build_handler_func(attr: TokenStream, input: TokenStream) -> TokenStream {
    let f: ItemFn = syn::parse2(input).expect("parse as proc_macro2::TokenStream");

    if f.sig.asyncness.is_none() {
        panic!("fn must be async to decorate");
    }

    let mut sig = f.sig.clone();
    sig.asyncness = None;
    sig.generics = syn::parse_quote!(<'a>);

    must_add_lifetime_to_response_writer(sig.inputs.first_mut().expect("miss ResponseWriter"));

    sig.output = syn::parse_quote!(
        -> ::core::pin::Pin<Box<::core::future::Future<Output = ()> + ::core::marker::Send + 'a>>
    );

    let args_name: Vec<Ident> = sig
        .inputs
        .iter()
        .map(|v| match v {
            FnArg::Typed(w) => {
                if let Pat::Ident(w) = w.pat.as_ref() {
                    return w.ident.clone();
                }
                panic!("missing identifier")
            }
            _ => panic!("unexpected arg"),
        })
        .collect();

    let func_name = f.sig.ident.clone();

    let out = quote! {
        #sig {
            #attr
            #f

            Box::pin(#func_name( #(#args_name),* ))
        }
    };

    out
}

fn must_add_lifetime_to_response_writer(arg: &mut FnArg) {
    let v = match arg {
        FnArg::Typed(v) => v,
        _ => panic!("no receiver is allowed"),
    };

    let ty = match v.ty.as_mut() {
        Type::Reference(r) => r,
        _ => panic!("bad type"),
    };
    ty.lifetime = Some(Lifetime::new("'a", Span::call_site()));
}

#[cfg(test)]
mod tests;
