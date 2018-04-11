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

use parser::Result as PResult;
use proc_macro::{Span, TokenStream};
use spanned::Spanned;

use ext::*;
use syn::*;

const NO_GENERICS: &str = "structs with generics cannot derive `UriDisplay`";
const ONLY_STRUCTS: &str = "`UriDisplay` can only be derived for structs";
const NO_EMPTY_STRUCTS: &str = "`UriDisplay` cannot be derived for empty struct";
const NO_UNIT_STRUCTS: &str = "`UriDisplay` can only be derived for unit structs";
const ONLY_ONE_UNNAMED: &str = "`UriDisplay` can be derived for tuple-like structs of length only 1";

fn validate_struct(data_struct: &DataStruct, input: &DeriveInput) -> PResult<()> {

    let fields = &data_struct.fields;

    match fields {
        Fields::Named(fields_named) => {},
        Fields::Unnamed(fields_unnamed) => {
            if fields_unnamed.unnamed.len() > 1 {
                return Err(input.span().error(ONLY_ONE_UNNAMED))
            }
        },
        Fields::Unit => return Err(input.span().error(NO_UNIT_STRUCTS))
    }

    // Reject empty structs.
    if fields.is_empty() {
        return Err(input.span().error(NO_EMPTY_STRUCTS))
    }

    Ok(())
}


fn real_derive_uri_display_value(input: TokenStream) -> PResult<TokenStream> {
    // Parse the input `TokenStream` as a `syn::DeriveInput`, an AST.
    let input: DeriveInput = syn::parse(input).map_err(|e| {
        Span::call_site().error(format!("error: failed to parse input: {:?}", e))
    })?;


    // This derive doesn't support generics. Error out if there are generics.
    if !input.generics.params.is_empty() {
        return Err(input.generics.span().error(NO_GENERICS));
    }

    let inp = &input;

    match inp.data {
        Data::Struct(ref data_struct) => {
            validate_struct(data_struct, &input)?;
            real_derive_uri_display_value_for_struct(data_struct, &input)
        },
        _ => Err(input.span().error(ONLY_STRUCTS))
    }
}

// Precondition: input must be valid non-unit struct
fn real_derive_uri_display_value_for_struct(
    data_struct: &DataStruct, input: &DeriveInput
) -> PResult<TokenStream> {

    let fields = &data_struct.fields;

    match fields {
        Fields::Named(fields_named) =>
            real_derive_uri_display_value_for_named_struct(&fields_named, data_struct, input),
        Fields::Unnamed(fields_unnamed) =>
            real_derive_uri_display_value_for_unnamed_struct(&fields_unnamed, data_struct, input),
        _ => panic!("This codepath is never reached.") // TODO: something better
    }
}

// Precondition: there is exactly one field in the struct
fn real_derive_uri_display_value_for_unnamed_struct(
    fields_unnamed: &FieldsUnnamed, data_struct: &DataStruct, input: &DeriveInput
) -> PResult<TokenStream> {

    let name = input.ident;
    let scope = Ident::from(format!("scope_{}", name.to_string().to_lowercase()));
    Ok(quote! {
        mod #scope {
            extern crate std;
            extern crate rocket;

            use self::std::prelude::v1::*;
            use self::std::fmt;
            use self::rocket::http::uri::*;

            impl UriDisplay for #name {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                  write!(f, "{}", &self.0 as &UriDisplay)
                }
            }
        }
    }.into())
}

fn real_derive_uri_display_value_for_named_struct(
    fields_named: &FieldsNamed, data_struct: &DataStruct, input: &DeriveInput
) -> PResult<TokenStream> {

    // Enumerate all the field names in the struct.
    let idents = fields_named.named.iter().map(|v| v.ident.as_ref().expect("named field"));
    // Generate format string.
    let format_string = fields_named.named.iter().map(|v| v.ident.as_ref().unwrap().to_string() + "={}")
                                                 .collect::<Vec<_>>()
                                                 .join("&");

    let name = input.ident;
    let scope = Ident::from(format!("scope_{}", name.to_string().to_lowercase()));
    // Generate the implementation.
    Ok(quote! {
        mod #scope {
            extern crate std;
            extern crate rocket;

            use self::std::prelude::v1::*;
            use self::std::fmt;
            use self::rocket::http::uri::*;

            impl UriDisplay for #name {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                  write!(f, #format_string, #(&self.#idents as &UriDisplay),*)
                }
            }
        }
    }.into())
}

#[proc_macro_derive(UriDisplay)]
pub fn derive_uri_display_value(input: TokenStream) -> TokenStream {
    real_derive_uri_display_value(input).unwrap_or_else(|diag| {
       diag.emit();
       TokenStream::empty()
    })
}
