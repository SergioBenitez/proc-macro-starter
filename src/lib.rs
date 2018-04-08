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
const ONLY_NAMED: &str = "`UriDisplay` can only be derived for named-field structs";
const EMPTY_STRUCT_WARN: &str = "deriving `UriDisplay` for empty struct";

fn validate_input(input: DeriveInput) -> PResult<FieldsNamed> {
    // This derive doesn't support generics. Error out if there are generics.
    if !input.generics.params.is_empty() {
        return Err(input.generics.span().error(NO_GENERICS));
    }

    // This derive only works for structs. Error out if the input is not a struct.
    let input_span = input.span();
    let data = input.data.into_struct().ok_or_else(|| input_span.error(ONLY_STRUCTS))?;

    // This derive only works for named-field structs.
    let named_fields = match data.fields {
        Fields::Named(fields) => fields,
        _ => return Err(input_span.error(ONLY_NAMED))
    };

    // Emit a warning if the struct is empty.
    if named_fields.named.is_empty() {
        Span::call_site().warning(EMPTY_STRUCT_WARN).emit();
    }

    Ok(named_fields)
}


fn real_derive_uri_display_value(input: TokenStream) -> PResult<TokenStream> {
    // Parse the input `TokenStream` as a `syn::DeriveInput`, an AST.
    let input: DeriveInput = syn::parse(input).map_err(|e| {
        Span::call_site().error(format!("error: failed to parse input: {:?}", e))
    })?;

    // Validate the struct.
    let name = input.ident;
    let struct_fields = validate_input(input)?;

    // Create iterators over the identifers as idents and as format strings.
    let idents = struct_fields.named.iter().map(|v| v.ident.unwrap());
    let format_strs = struct_fields.named.iter().map(|v| {
        let field = v.ident.unwrap().to_string();
        field + "={}"
    });

    // Generate the implementation.
    Ok(quote! {
        mod scope {
            extern crate std;
            extern crate rocket;

            use self::std::prelude::v1::*;
            use self::std::fmt;
            use self::rocket::http::uri::*;

            macro_rules! uri_format_helper {
                (&) => { "&" };
                ($x:tt) => { $x };
            }
            macro_rules! uri_format {
                ($($x:tt)*) => {
                    concat!($(uri_format_helper!($x)),*)
                };
            }

            impl UriDisplay for #name {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                  write!(f, uri_format!(#(#format_strs)&*), #(&self.#idents as &UriDisplay),*)
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
