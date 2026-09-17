//! `#[derive(ChoiceCriteria)]`: implements `systemone::ChoiceCriteria` for a unit-variant
//! enum, generating the outbound `choice` labels/descriptions from the enum's variants and
//! doc comments at compile time.
//!
//! Labels follow `#[serde(rename = "...")]` / `#[serde(rename_all = "...")]` when present
//! (a subset: lowercase, UPPERCASE, snake_case, SCREAMING_SNAKE_CASE, kebab-case,
//! SCREAMING-KEBAB-CASE, camelCase, PascalCase), so the same enum can also derive
//! `serde::Deserialize` for the response side without the two disagreeing. With no
//! `rename_all`, labels default to the variant name lowercased.
//!
//! See `systemone-facet` for the same goal via runtime reflection instead.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(ChoiceCriteria, attributes(serde))]
pub fn derive_choice_criteria(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input,
            "ChoiceCriteria can only be derived for enums",
        ));
    };

    let rename_all = container_rename_all(&input.attrs)?;
    let name = &input.ident;

    let mut entries = Vec::with_capacity(data.variants.len());
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "ChoiceCriteria only supports unit variants",
            ));
        }
        let label = variant_rename(&variant.attrs)?
            .unwrap_or_else(|| apply_case(&variant.ident.to_string(), rename_all.as_deref()));
        let description = match variant_doc(&variant.attrs) {
            Some(text) => {
                quote! { ::std::option::Option::Some(::systemone::EntryType::from(#text)) }
            }
            None => quote! { ::std::option::Option::None },
        };
        entries.push(quote! { (#label, #description) });
    }

    Ok(quote! {
        impl ::systemone::ChoiceCriteria for #name {
            fn choice_criteria() -> ::std::vec::Vec<(&'static str, ::std::option::Option<::systemone::EntryType>)> {
                ::std::vec![ #(#entries),* ]
            }
        }
    })
}

/// `#[serde(...)]` `key = "value"` pairs on an item, ignoring bare-path and non-string entries.
fn serde_name_values(attrs: &[syn::Attribute]) -> syn::Result<Vec<syn::MetaNameValue>> {
    let mut out = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let nested = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        )?;
        out.extend(nested.into_iter().filter_map(|meta| match meta {
            syn::Meta::NameValue(nv) => Some(nv),
            _ => None,
        }));
    }
    Ok(out)
}

fn string_lit(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) => Some(s.value()),
        _ => None,
    }
}

fn container_rename_all(attrs: &[syn::Attribute]) -> syn::Result<Option<String>> {
    Ok(serde_name_values(attrs)?
        .into_iter()
        .find(|nv| nv.path.is_ident("rename_all"))
        .and_then(|nv| string_lit(&nv.value)))
}

fn variant_rename(attrs: &[syn::Attribute]) -> syn::Result<Option<String>> {
    Ok(serde_name_values(attrs)?
        .into_iter()
        .find(|nv| nv.path.is_ident("rename"))
        .and_then(|nv| string_lit(&nv.value)))
}

/// Doc comment lines, trimmed and joined with a space; `None` if there are none.
fn variant_doc(attrs: &[syn::Attribute]) -> Option<String> {
    let lines: Vec<String> = attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| match &a.meta {
            syn::Meta::NameValue(nv) => string_lit(&nv.value),
            _ => None,
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    (!lines.is_empty()).then(|| lines.join(" "))
}

fn apply_case(name: &str, rename_all: Option<&str>) -> String {
    match rename_all {
        Some("lowercase") => name.to_lowercase(),
        Some("UPPERCASE") => name.to_uppercase(),
        Some("PascalCase") => name.to_string(),
        Some("camelCase") => to_camel_case(name),
        Some("snake_case") => to_snake_case(name),
        Some("SCREAMING_SNAKE_CASE") => to_snake_case(name).to_uppercase(),
        Some("kebab-case") => to_snake_case(name).replace('_', "-"),
        Some("SCREAMING-KEBAB-CASE") => to_snake_case(name).to_uppercase().replace('_', "-"),
        // No rename_all: default to lowercase, matching systemone-facet's default so the
        // two backends are directly comparable for a plain, unannotated enum.
        _ => name.to_lowercase(),
    }
}

fn to_snake_case(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn to_camel_case(name: &str) -> String {
    let snake = to_snake_case(name);
    let mut out = String::new();
    for (i, part) in snake.split('_').filter(|p| !p.is_empty()).enumerate() {
        if i == 0 {
            out.push_str(part);
            continue;
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_case_is_lowercase() {
        assert_eq!(apply_case("Billing", None), "billing");
    }

    #[test]
    fn snake_case_conversion() {
        assert_eq!(to_snake_case("TechnicalIssue"), "technical_issue");
    }

    #[test]
    fn camel_case_conversion() {
        assert_eq!(to_camel_case("TechnicalIssue"), "technicalIssue");
    }

    #[test]
    fn kebab_case_via_apply_case() {
        assert_eq!(
            apply_case("TechnicalIssue", Some("kebab-case")),
            "technical-issue"
        );
    }
}
