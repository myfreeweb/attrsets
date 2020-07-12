use itertools::Itertools;
use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::iter::FromIterator;

#[derive(Clone, Copy)]
struct Ctx<'a> {
    all_variants: &'a [String],
    cur_variant: Option<&'a str>,
}

fn filter_field(ctx: Ctx, field: syn::Field) -> syn::Field {
    syn::Field {
        attrs: field
            .attrs
            .into_iter()
            .flat_map(|a| {
                assert!(a.style == syn::AttrStyle::Outer);
                if a.path.is_ident("attrset") {
                    if let Some(TokenTree::Group(g)) = a.tokens.into_iter().next() {
                        assert!(g.delimiter() == Delimiter::Parenthesis);
                        let mut tokens = g.stream().into_iter();
                        let on_variants = tokens
                            .take_while_ref(|t| match t {
                                TokenTree::Punct(p) if p.as_char() == ',' => true,
                                TokenTree::Ident(i)
                                    if i.to_string() == "_"
                                        || ctx.all_variants.iter().any(|v| *v == i.to_string()) =>
                                {
                                    true
                                }
                                _ => false,
                            })
                            .flat_map(|t| match t {
                                TokenTree::Punct(p) if p.as_char() == ',' => None,
                                TokenTree::Ident(i) => Some(i.to_string()),
                                _ => unreachable!(),
                            })
                            .collect::<Vec<_>>();
                        let v_matches = if let Some(v) = ctx.cur_variant {
                            on_variants.iter().any(|vv| vv == v)
                        } else {
                            false
                        };
                        let plain_matches =
                            ctx.cur_variant.is_none() && on_variants.iter().any(|vv| vv == "_");
                        if v_matches || plain_matches {
                            let path = syn::parse2::<syn::Path>(TokenStream::from_iter(
                                tokens.take_while_ref(|t| match t {
                                    TokenTree::Punct(p) if p.as_char() == ':' => true,
                                    TokenTree::Ident(_) => true,
                                    _ => false,
                                }),
                            ))
                            .unwrap();
                            Some(syn::Attribute {
                                tokens: TokenStream::from_iter(tokens),
                                path,
                                ..a
                            })
                        } else {
                            None
                        }
                    } else {
                        panic!("attrset attr should look like attrset(...)");
                    }
                } else {
                    Some(a)
                }
            })
            .collect(),
        ..field
    }
}

fn filter_fields(ctx: Ctx, fields: syn::Fields) -> syn::Fields {
    match fields {
        syn::Fields::Named(n) => syn::Fields::Named(syn::FieldsNamed {
            named: n
                .named
                .into_pairs()
                .map(|p| match p {
                    syn::punctuated::Pair::Punctuated(f, c) => {
                        syn::punctuated::Pair::Punctuated(filter_field(ctx, f), c)
                    }
                    syn::punctuated::Pair::End(f) => {
                        syn::punctuated::Pair::End(filter_field(ctx, f))
                    }
                })
                .collect(),
            ..n
        }),
        syn::Fields::Unnamed(u) => syn::Fields::Unnamed(syn::FieldsUnnamed {
            unnamed: u
                .unnamed
                .into_pairs()
                .map(|p| match p {
                    syn::punctuated::Pair::Punctuated(f, c) => {
                        syn::punctuated::Pair::Punctuated(filter_field(ctx, f), c)
                    }
                    syn::punctuated::Pair::End(f) => {
                        syn::punctuated::Pair::End(filter_field(ctx, f))
                    }
                })
                .collect(),
            ..u
        }),
        syn::Fields::Unit => syn::Fields::Unit,
    }
}

fn filter_def(ctx: Ctx, inp: syn::DeriveInput) -> syn::DeriveInput {
    let data = match inp.data {
        syn::Data::Struct(stru) => syn::Data::Struct(syn::DataStruct {
            fields: filter_fields(ctx, stru.fields),
            ..stru
        }),
        syn::Data::Enum(enu) => syn::Data::Enum(syn::DataEnum {
            variants: enu
                .variants
                .into_pairs()
                .map(|p| match p {
                    syn::punctuated::Pair::Punctuated(v, c) => syn::punctuated::Pair::Punctuated(
                        syn::Variant {
                            fields: filter_fields(ctx, v.fields),
                            ..v
                        },
                        c,
                    ),
                    syn::punctuated::Pair::End(v) => syn::punctuated::Pair::End(syn::Variant {
                        fields: filter_fields(ctx, v.fields),
                        ..v
                    }),
                })
                .collect(),
            ..enu
        }),
        syn::Data::Union(_) => panic!("attrsets does not support union"),
    };
    syn::DeriveInput {
        ident: Ident::new(
            &format!("{}{}", inp.ident.to_string(), ctx.cur_variant.unwrap_or("")),
            Span::call_site(),
        ),
        data,
        ..inp
    }
}

#[proc_macro_attribute]
pub fn attrsets(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_ast: syn::DeriveInput = syn::parse(item).unwrap();

    let all_variants = attr
        .into_iter()
        .flat_map(|t| match t {
            proc_macro::TokenTree::Punct(p) if p.as_char() == ',' => None,
            proc_macro::TokenTree::Ident(i) => Some(i.to_string()),
            _ => panic!("attrsets attr: bad token: {}", t),
        })
        .collect::<Vec<_>>();

    let mut tst = filter_def(
        Ctx {
            all_variants: &all_variants,
            cur_variant: None,
        },
        item_ast.clone(),
    )
    .into_token_stream();

    for v in all_variants.iter() {
        tst.extend(
            filter_def(
                Ctx {
                    all_variants: &all_variants,
                    cur_variant: Some(v),
                },
                item_ast.clone(),
            )
            .into_token_stream(),
        );
    }

    tst.into()
}
