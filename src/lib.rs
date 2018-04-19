#![feature(proc_macro, core_intrinsics, decl_macro)]
#![recursion_limit="256"]

extern crate syn;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use] extern crate quote;
extern crate rocket;

mod parser;
mod spanned;
mod ext;
mod codegen_ext;

use parser::Result as PResult;
use proc_macro::{Span, TokenStream};
use spanned::Spanned;

use ext::*;
use syn::*;
use codegen_ext::*;
use quote::Tokens;

const NO_NON_LIFETIME_GENERICS: &str = "`UriDisplay` cannot be derived for non-lifetime generics";
const NO_UNIONS: &str = "unions cannot derive `UriDisplay`";
const NO_EMPTY_FIELDS: &str = "`UriDisplay` cannot be derived for structs or variants with no fields";
const NO_NULLARY: &str = "`UriDisplay` cannot only be derived for nullary structs and enum variants";
const NO_EMPTY_ENUMS: &str = "`UriDisplay` cannot only be derived for enums with no variants";
const ONLY_ONE_UNNAMED: &str = "`UriDisplay` can be derived for tuple-like structs of length only 1";

fn validate_fields(fields: &Fields) -> PResult<()> {

    match fields {
        Fields::Named(_) => {},
        Fields::Unnamed(fields_unnamed) => {
            if fields_unnamed.unnamed.len() > 1 {
                return Err(fields.span().error(ONLY_ONE_UNNAMED))
            }
        },
        Fields::Unit => return Err(fields.span().error(NO_NULLARY))
    }

    // Reject empty structs.
    if fields.is_empty() {
        return Err(fields.span().error(NO_EMPTY_FIELDS))
    }

    Ok(())
}

fn validate_struct(data_struct: &DataStruct, input: &DeriveInput) -> PResult<()> {
    validate_fields(&data_struct.fields)
}

fn validate_enum(data_enum: &DataEnum, input: &DeriveInput) -> PResult<()> {
    if data_enum.variants.len() == 0 {
        return Err(input.span().error(NO_EMPTY_ENUMS));
    }
    for variant in data_enum.variants.iter() {
        validate_fields(&variant.fields)?;
    }
    Ok(())
}

fn real_derive_uri_display_value(input: TokenStream) -> PResult<TokenStream> {
    // Parse the input `TokenStream` as a `syn::DeriveInput`, an AST.
    let input: DeriveInput = syn::parse(input).map_err(|e| {
        Span::call_site().error(format!("error: failed to parse input: {:?}", e))
    })?;


    // This derive doesn't support non-lifetime generics.
    for param in input.generics.params.iter() {
        match param {
            GenericParam::Lifetime(_) => { },
            _ => return Err(param.span().error(NO_NON_LIFETIME_GENERICS))
        }
    }

    match input.data {
        Data::Struct(ref data_struct) => {
            validate_struct(data_struct, &input)?;
            real_derive_uri_display_value_for_struct(data_struct, &input)
        },
        Data::Enum(ref data_enum) => {
            validate_enum(data_enum, &input)?;
            real_derive_uri_display_value_for_enums(data_enum, &input)
        },
        _ => return Err(input.span().error(NO_UNIONS))
    }
}

// Precondition: input must be valid enum
fn real_derive_uri_display_value_for_enums(
    data_enum: &DataEnum, input: &DeriveInput
) -> PResult<TokenStream> {

    let variants = &data_enum.variants;
    let variant_idents = variants.iter().map(|v| v.ident);
    let variant_fields = variants.iter().map(|v| v.fields.ref_match_tokens());
    let variant_match_bodies = variants.iter().map(|v| fields_to_fmt_body(&v.fields, FieldOrigin::Variant));
    let name_repeated = ::std::iter::repeat(input.ident);

    let body = quote! {
        match *self {
            #(#name_repeated::#variant_idents #variant_fields => {
                #variant_match_bodies
            }),*
        }
    };

    Ok(wrap_in_fmt_and_impl(body, input).into())
}

// Precondition: input must be valid struct
fn real_derive_uri_display_value_for_struct(
    data_struct: &DataStruct, input: &DeriveInput
) -> PResult<TokenStream> {

    let fmt_body = fields_to_fmt_body(&data_struct.fields, FieldOrigin::Struct);
    Ok(wrap_in_fmt_and_impl(fmt_body, input).into())
}

fn fields_to_fmt_body(fields: &Fields, origin: FieldOrigin) -> Tokens {
    let vars = fields.iter().enumerate().map(|(i, field)| field.to_variable_tokens(i, origin));

    match fields {
        Fields::Named(ref fields_named) => {
            let names = fields_named.named.iter().map(|field| field.ident.as_ref().unwrap().to_string());
            quote! {
                #(f.with_prefix(#names, |mut _f| _UriDisplay::fmt(&#vars, &mut _f) )?;)*
                Ok(())
            }
        },
        Fields::Unnamed(_) => {
            quote! {
                #(_UriDisplay::fmt(&#vars, f)?;)*
                Ok(())
            }
        },
        _ => panic!("This code path is never reached!")
    }
}

fn wrap_in_fmt_and_impl(tokens: Tokens, input: &DeriveInput) -> Tokens {
    wrap_in_impl(wrap_in_fmt(tokens), input)
}

fn wrap_in_fmt(tokens: Tokens) -> Tokens {
    quote! {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            #tokens
        }
    }
}

fn wrap_in_impl(tokens: Tokens, input: &DeriveInput) -> Tokens {
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let scope = Ident::from(format!("scope_{}", name.to_string().to_lowercase()));

    quote! {
        mod #scope {
            extern crate std;
            extern crate rocket;

            use self::std::prelude::v1::*;
            use self::std::fmt;
            use self::rocket::http::uri::*;

            impl #impl_generics _UriDisplay for #name #ty_generics #where_clause {
                #tokens
            }
        }
    }
}

#[proc_macro_derive(_UriDisplay)]
pub fn derive_uri_display_value(input: TokenStream) -> TokenStream {
    real_derive_uri_display_value(input).unwrap_or_else(|diag| {
       diag.emit();
       TokenStream::empty()
    })
}
