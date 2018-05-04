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
mod uri_codegen;

use parser::Result as PResult;
use proc_macro::{Span, TokenStream};
use spanned::Spanned;

use ext::*;
use syn::*;
use uri_codegen::*;

const NO_CONST_GENERICS: &str = "`UriDisplay` cannot be derived for const generics";
const NO_UNIONS: &str = "unions cannot derive `UriDisplay`";
const NO_EMPTY_FIELDS: &str = "`UriDisplay` cannot be derived for structs or variants with no fields";
const NO_NULLARY: &str = "`UriDisplay` cannot only be derived for nullary structs and enum variants";
const NO_EMPTY_ENUMS: &str = "`UriDisplay` cannot only be derived for enums with no variants";
const ONLY_ONE_UNNAMED: &str = "`UriDisplay` can be derived for tuple-like structs of length only 1";

fn validate_fields(fields: &Fields, parent_span: Span) -> PResult<()> {
    // Reject empty structs and variants.
    if fields.is_empty() {
        return Err(parent_span.error(NO_EMPTY_FIELDS))
    }

    match fields {
        Fields::Unnamed(ref u_fields) if u_fields.unnamed.len() > 1 => {
            Err(u_fields.unnamed.span().error(ONLY_ONE_UNNAMED))
        },
        Fields::Unit => Err(parent_span.error(NO_NULLARY)),
        _ => Ok(())
    }
}

fn validate_struct(data_struct: &DataStruct, input: &DeriveInput) -> PResult<()> {
    validate_fields(&data_struct.fields, input.span())
}

fn validate_enum(data_enum: &DataEnum, input: &DeriveInput) -> PResult<()> {
    if data_enum.variants.len() == 0 {
        return Err(input.span().error(NO_EMPTY_ENUMS));
    }
    for variant in data_enum.variants.iter() {
        validate_fields(&variant.fields, variant.span())?;
    }
    Ok(())
}

fn real_derive_uri_display_value(input: TokenStream) -> PResult<TokenStream> {
    // Parse the input `TokenStream` as a `syn::DeriveInput`, an AST.
    let input: DeriveInput = syn::parse(input).map_err(|e| {
        Span::call_site().error(format!("error: failed to parse input: {:?}", e))
    })?;


    // This derive doesn't support const generics.
    for param in input.generics.params.iter() {
        match param {
            GenericParam::Const(_) => return Err(param.span().error(NO_CONST_GENERICS)),
            _ => { }
        }
    }

    // Validate input, parse into internal AST, and generate code for impl
    let tokens = match input.data {
        Data::Struct(ref data_struct) => {
            validate_struct(data_struct, &input)?;
            let struct_node = StructNode::new(data_struct, &input.ident, &input.generics);
            quote!(#struct_node)
        },
        Data::Enum(ref data_enum) => {
            validate_enum(data_enum, &input)?;
            let enum_node = EnumNode::new(data_enum, &input.ident, &input.generics);
            quote!(#enum_node)
        },
        _ => return Err(input.span().error(NO_UNIONS))
    };

    Ok(tokens.into())   
}

#[proc_macro_derive(_UriDisplay)]
pub fn derive_uri_display_value(input: TokenStream) -> TokenStream {
    real_derive_uri_display_value(input).unwrap_or_else(|diag| {
       diag.emit();
       TokenStream::empty()
    })
}
