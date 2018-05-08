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

const DB_ATTR_NO_FIELDS_ERR: &str = "`database` attribute cannot be applied to unit structs";
const DB_ATTR_NOT_TUPLE_STRUCT_ERR: &str = "`database` attribute can only be applied to tuple structs";
const DB_ATTR_NO_GENERICS: &str = "`database` attribute cannot be applied to structs with generic types";
const DB_ATTR_ONLY_STRUCTS: &str = "`database` attribute can only be applied to structs";

#[derive(Debug, Clone)]
pub(crate) struct FieldMember<'f> {
    field: &'f Field,
    member: Member
}

fn validate_database_input(input: DeriveInput) -> PResult<DataStruct> {
    if !input.generics.params.is_empty() {
        return Err(input.generics.span().error(DB_ATTR_NO_GENERICS));
    }

    let input_span = input.span();
    let data_struct = input.data.into_struct().ok_or_else(|| input_span.error(DB_ATTR_ONLY_STRUCTS))?;

    match data_struct.fields {
        Fields::Named(_) => return Err(data_struct.fields.span().error(DB_ATTR_NOT_TUPLE_STRUCT_ERR)),
        _ => {},
    };

    if data_struct.fields.is_empty() {
        return Err(Span::call_site().error(DB_ATTR_NO_FIELDS_ERR));
    }

    Ok(data_struct)
}

// TODO: Get the attribute for the database name
// TODO: Get the inner type for the database connection
fn apply_database_attr(input: TokenStream) -> PResult<TokenStream> {
    let input: DeriveInput = syn::parse(input).map_err(|e| {
        Span::call_site().error(format!("error: failed to parse input: {:?}", e))
    })?;

    let name = input.ident;
    let struct_data = validate_database_input(input)?;

    // TODO: The proc macro

    Ok(quote! {
        mod scope {
        }
    }.into())
}

#[proc_macro_attribute]
pub fn database(metadata: TokenStream, input: TokenStream) -> TokenStream {
    println!("{:#?}", metadata);
    println!("{:#?}", input);
    apply_database_attr(input).unwrap_or_else(|diagnostic| {
        diagnostic.emit();
        TokenStream::empty()
    })
}
