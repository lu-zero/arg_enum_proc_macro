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
use std::iter::FromIterator;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Lit::Str;
use syn::Meta::NameValue;
use syn::{
    parenthesized, parse2, parse_macro_input, Data, DeriveInput, Ident, LitStr, MetaNameValue,
    Token,
};

#[derive(Debug)]
enum ArgEnumAttr {
    /// An alias for the Enum
    Alias(Literal),
    /// Override the default string representation for the variant
    Name(Literal),
}

impl Parse for ArgEnumAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        use self::ArgEnumAttr::*;
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;

            match name_str.as_ref() {
                "alias" => {
                    let alias: LitStr = input.parse()?;
                    Ok(Alias(Literal::string(&alias.value())))
                }
                "name" => {
                    let name: LitStr = input.parse()?;
                    Ok(Name(Literal::string(&name.value())))
                }
                _ => panic!("unexpected attribute {}", name_str),
            }
        } else {
            panic!("unexpected attribute: {}", name_str)
        }
    }
}

#[derive(Debug)]
struct ArgEnumAttrs {
    paren_token: syn::token::Paren,
    attrs: Punctuated<ArgEnumAttr, Token![,]>,
}

impl Parse for ArgEnumAttrs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;

        Ok(ArgEnumAttrs {
            paren_token: parenthesized!(content in input),
            attrs: content.parse_terminated(ArgEnumAttr::parse)?,
        })
    }
}

/// Implement [`std::fmt::Display`], [`std::str::FromStr`] and `variants()`.
///
/// The invocation:
/// ``` no_run
/// use arg_enum_proc_macro::ArgEnum;
///
/// #[derive(ArgEnum)]
/// enum Foo {
///     A,
///     /// Describe B
///     #[arg_enum(alias = "Bar")]
///     B,
///     /// Describe C
///     /// Multiline
///     #[arg_enum(name = "Baz")]
///     C,
/// }
/// ```
///
/// produces:
/// ``` no_run
/// enum Foo {
///     A,
///     B,
///     C
/// }
/// impl ::std::str::FromStr for Foo {
///     type Err = String;
///
///     fn from_str(s: &str) -> ::std::result::Result<Self,Self::Err> {
///         match s {
///             "A" | _ if s.eq_ignore_ascii_case("A") => Ok(Foo::A),
///             "B" | _ if s.eq_ignore_ascii_case("B") => Ok(Foo::B),
///             "Bar" | _ if s.eq_ignore_ascii_case("Bar") => Ok(Foo::B),
///             "Baz" | _ if s.eq_ignore_ascii_case("Baz") => Ok(Foo::C),
///             _ => Err({
///                 let v = vec![ "A", "B", "Bar", "Baz" ];
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
///             Foo::C => write!(f, "C"),
///         }
///     }
/// }
///
/// impl Foo {
///     /// Returns an array of valid values which can be converted into this enum.
///     #[allow(dead_code)]
///     pub fn variants() -> [&'static str; 4] {
///         [ "A", "B", "Bar", "Baz", ]
///     }
///     #[allow(dead_code)]
///     pub fn descriptions() -> [(&'static [&'static str], &'static [&'static str]) ;3] {
///         [(&["A"], &[]),
///          (&["B", "Bar"], &[" Describe B"]),
///          (&["Baz"], &[" Describe C", " Multiline"]),]
///     }
/// }
/// ```
#[proc_macro_derive(ArgEnum, attributes(arg_enum))]
pub fn arg_enum(items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(items as DeriveInput);

    let name = input.ident;
    let variants = if let Data::Enum(data) = input.data {
        data.variants
    } else {
        panic!("Only enum supported");
    };

    let all_variants: Vec<(TokenTree, &Ident)> = variants
        .iter()
        .flat_map(|item| {
            let id = &item.ident;
            if !item.fields.is_empty() {
                panic!(
                    "Only enum with unit variants are supported! \n\
                    Variant {}::{} is not an unit variant",
                    name,
                    &id.to_string()
                );
            }

            let lit: TokenTree = Literal::string(&id.to_string()).into();
            let mut all_lits = vec![(lit, id)];
            item.attrs
                .iter()
                .filter(|attr| attr.path.is_ident("arg_enum"))
                // .flat_map(|attr| {
                .for_each(|attr| {
                    let attrs: ArgEnumAttrs = parse2(attr.tokens.clone()).unwrap();

                    for attr in attrs.attrs {
                        match attr {
                            ArgEnumAttr::Alias(alias) => all_lits.push((alias.into(), id)),
                            ArgEnumAttr::Name(name) => all_lits[0] = (name.into(), id),
                        }
                    }
                });
            all_lits.into_iter()
        })
        .collect();

    let len = all_variants.len();

    let from_str_match = all_variants.iter().flat_map(|(lit, id)| {
        let pat: TokenStream = quote! {
            #lit | _ if s.eq_ignore_ascii_case(#lit) => Ok(#name::#id),
        }
        .into();

        pat.into_iter()
    });

    let from_str_match = TokenStream::from_iter(from_str_match);

    let all_descriptions: Vec<(Vec<TokenTree>, Vec<LitStr>)> = variants
        .iter()
        .map(|item| {
            let id = &item.ident;
            let description = item
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("doc"))
                .filter_map(|attr| {
                    if let Ok(NameValue(MetaNameValue { lit: Str(s), .. })) = attr.parse_meta() {
                        Some(s)
                    } else {
                        // non #[doc = "..."] attributes are not our concern
                        // we leave them for rustc to handle
                        None
                    }
                })
                .collect();
            let lit: TokenTree = Literal::string(&id.to_string()).into();
            let mut all_names = vec![lit];
            item.attrs
                .iter()
                .filter(|attr| attr.path.is_ident("arg_enum"))
                // .flat_map(|attr| {
                .for_each(|attr| {
                    let attrs: ArgEnumAttrs = parse2(attr.tokens.clone()).unwrap();

                    for attr in attrs.attrs {
                        match attr {
                            ArgEnumAttr::Alias(alias) => all_names.push(alias.into()),
                            ArgEnumAttr::Name(name) => all_names[0] = name.into(),
                        }
                    }
                });

            (all_names, description)
        })
        .collect();

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

    let comma: TokenTree = Punct::new(',', proc_macro2::Spacing::Alone).into();
    let array_items = all_variants
        .iter()
        .flat_map(|(tok, _id)| vec![tok.clone(), comma.clone()].into_iter());

    let array_items = TokenStream::from_iter(array_items);

    let array_descriptions = all_descriptions.iter().map(|(names, descr)| {
        quote! {
            (&[ #(#names),* ], &[ #(#descr),* ]),
        }
    });
    let array_descriptions = TokenStream::from_iter(array_descriptions);

    let len_descriptions = all_descriptions.len();

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
            #[allow(dead_code)]
            /// Returns an array of valid values which can be converted into this enum.
            pub fn variants() -> [&'static str; #len] {
                [ #array_items ]
            }
            #[allow(dead_code)]
            /// Returns an array of touples (variants, description)
            pub fn descriptions() -> [(&'static [&'static str], &'static [&'static str]); #len_descriptions] {
                [ #array_descriptions ]
            }
        }
    }
    .into();

    ret.into()
}
