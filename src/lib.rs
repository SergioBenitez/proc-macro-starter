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

const DB_CONN_NO_FIELDS_ERR: &str = "`DbConn` cannot be derived for unit structs";
const DB_CONN_NOT_TUPLE_STRUCT_ERR: &str = "`DbConn` can only be derived for tuple structs";
const DB_CONN_NO_GENERICS: &str = "structs with generics cannot derive `DbConn`";
const DB_CONN_ONLY_STRUCTS: &str = "`DbConn` can only be derived for structs";
const DB_CONN_NO_CONNECTION_SPECIFIED: &str = "`DbConn` derive requires #[connection_name = \"...\"] attribute";

#[derive(Debug, Clone)]
pub(crate) struct FieldMember<'f> {
    field: &'f Field,
    member: Member
}

fn validate_db_conn_input(input: DeriveInput) -> PResult<DataStruct> {
    if !input.generics.params.is_empty() {
        return Err(input.generics.span().error(DB_CONN_NO_GENERICS));
    }

    let input_span = input.span();
    let data_struct = input.data.into_struct().ok_or_else(|| input_span.error(DB_CONN_ONLY_STRUCTS))?;

    match data_struct.fields {
        Fields::Named(_) => return Err(data_struct.fields.span().error(DB_CONN_NOT_TUPLE_STRUCT_ERR)),
        _ => {},
    };

    if data_struct.fields.is_empty() {
        return Err(Span::call_site().error(DB_CONN_NO_FIELDS_ERR));
    }

    Ok(data_struct)
}

// TODO: Get the attribute for the database name
// TODO: Get the inner type for the database connection
fn real_derive_db_conn(input: TokenStream) -> PResult<TokenStream> {
    let input: DeriveInput = syn::parse(input).map_err(|e| {
        Span::call_site().error(format!("error: failed to parse input: {:?}", e))
    })?;

    let name = input.ident;
    let struct_data = validate_db_conn_input(input)?;

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
    real_derive_db_conn(input).unwrap_or_else(|diagnostic| {
        diagnostic.emit();
        TokenStream::empty()
    })
}
