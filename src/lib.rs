//! # arg_enum_proc_macro
//!
//! This crate consists in a procedural macro derive that provides the
//! same implementations that clap the [`clap::arg_enum`][1] macro provides:
//! [`std::fmt::Display`], [`std::str::FromStr`] and a `variants()` function.
//!
//! By using a procedural macro it allows documenting the enum fields
//! correctly and avoids the requirement of expanding the macro to use
//! the structure with [cbindgen](https://crates.io/crates/cbindgen).
//!
//! [1]: https://docs.rs/clap/2.32.0/clap/macro.arg_enum.html
//!

#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro2::{Literal, Punct, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, Data, DeriveInput};
use std::iter::FromIterator;

/// Implement [`std::fmt::Display`], [`std::str::FromStr`] and `variants()`.
///
/// The invocation:
/// ``` no_run
/// use arg_enum_proc_macro::ArgEnum;
///
/// #[derive(ArgEnum)]
/// enum Foo {
///     A,
///     B
/// }
/// ```
///
/// produces:
/// ``` no_run
/// enum Foo {
///     A,
///     B
/// }
/// impl ::std::str::FromStr for Foo {
///     type Err = String;
///
///     fn from_str(s: &str) -> ::std::result::Result<Self,Self::Err> {
///         match s {
///             "A" | _ if s.eq_ignore_ascii_case("A") => Ok(Foo::A),
///             "B" | _ if s.eq_ignore_ascii_case("B") => Ok(Foo::B),
///             _ => Err({
///                 let v = vec![ "A", "B" ];
///                 format!("valid values: {}", v.join(" ,"))
///             }),
///         }
///     }
/// }
/// impl ::std::fmt::Display for Foo {
///     fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
///         match *self {
///             Foo::A => write!(f, "A"),
///             Foo::B => write!(f, "B"),
///         }
///     }
/// }
///
/// impl Foo {
///     #[allow(dead_code)]
///     pub fn variants() -> [&'static str; 2] {
///         [ "A", "B" ]
///     }
/// }
/// ```
#[proc_macro_derive(ArgEnum)]
pub fn arg_enum(items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(items as DeriveInput);

    let name = input.ident;
    let variants = if let Data::Enum(data) = input.data {
        data.variants
    } else {
        panic!("Only enum supported");
    };

    let len = variants.len();

    let from_str_match = variants.iter().flat_map(|item| {
        let id = &item.ident;
        let lit: TokenTree = Literal::string(&id.to_string()).into();

        let pat: TokenStream = quote! {
            #lit | _ if s.eq_ignore_ascii_case(#lit) => Ok(#name::#id),
        }
        .into();

        pat.into_iter()
    });

    let from_str_match = TokenStream::from_iter(from_str_match);

    let display_match = variants.iter().flat_map(|item| {
        let id = &item.ident;
        let lit: TokenTree = Literal::string(&id.to_string()).into();

        let pat: TokenStream = quote! {
            #name::#id => write!(f, #lit),
        }
        .into();

        pat.into_iter()
    });

    let display_match = TokenStream::from_iter(display_match);

    let array_items = variants.iter().flat_map(|item| {
        let tok: TokenTree = Literal::string(&item.ident.to_string()).into();
        let comma: TokenTree = Punct::new(',', proc_macro2::Spacing::Alone).into();

        vec![tok, comma].into_iter()
    });

    let array_items = TokenStream::from_iter(array_items);

    let ret: TokenStream = quote_spanned! {
        Span::call_site() =>
        impl ::std::str::FromStr for #name {
            type Err = String;

            fn from_str(s: &str) -> ::std::result::Result<Self,Self::Err> {
                match s {
                    #from_str_match
                    _ => {
                        let values = vec![ #array_items ];

                        Err(format!("valid values: {}", values.join(" ,")))
                    }
                }
            }
        }
        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match *self {
                    #display_match
                }
            }
        }
        impl #name {
            pub fn variants() -> [&'static str; #len] {
                [ #array_items ]
            }
        }
    }
    .into();

    ret.into()
}
