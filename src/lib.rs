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

const NO_GENERICS: &str = "structs with generics cannot derive `UriDisplay`";
const NO_UNIONS: &str = "unions cannot derive `UriDisplay`";
const NO_EMPTY_FIELDS: &str = "`UriDisplay` cannot be derived for structs or variants with no fields";
const NO_NULLARY: &str = "`UriDisplay` cannot only be derived for nullary structs and enum variants";
const NO_EMPTY_ENUMS: &str = "`UriDisplay` cannot only be derived for enums with no variants";
const ONLY_ONE_UNNAMED: &str = "`UriDisplay` can be derived for tuple-like structs of length only 1";


#[derive(Debug, Clone)]
pub(crate) struct FieldMember<'f> {
    field: &'f Field,
    member: Member
}

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


    // This derive doesn't support generics. Error out if there are generics.
    if !input.generics.params.is_empty() {
        return Err(input.generics.span().error(NO_GENERICS));
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

    let name = input.ident;
    let scope = Ident::from(format!("scope_{}", name.to_string().to_lowercase()));
    let variants = &data_enum.variants;
    let variant_idents = variants.iter().map(|v| v.ident);
    let variant_fields = variants.iter().map(|v| v.fields.ref_match_tokens());

    let variant_match_bodies = variants.iter().map(|v| {
        let match_field_idents = v.fields.iter().enumerate().map(field_to_ref);
        match v.fields {
            Fields::Unnamed(_) => {
                quote! {
                    #(_UriDisplay::fmt(#match_field_idents, f)?;)*
                    Ok(())
                }
            },
            Fields::Named(_) => {
                let field_ident_strs = v.fields.iter().map(|f| f.ident.unwrap().to_string());
                quote! {
                    #(f.with_prefix(#field_ident_strs, |_f| _UriDisplay::fmt(#match_field_idents, _f))?;)*
                    Ok(())
                }
            },
            Fields::Unit => panic!("This code path is never reached")
        }
    });

    let name_repeated = ::std::iter::repeat(name);

    // Generate the implementation.
    Ok(quote! {
        mod #scope {
            extern crate std;
            extern crate rocket;

            use self::std::prelude::v1::*;
            use self::std::fmt;
            use self::rocket::http::uri::*;

            impl _UriDisplay for #name {
                fn fmt(&self, f: &mut UriFormatter) -> fmt::Result {
                    match *self {
                        #(#name_repeated::#variant_idents #variant_fields => {
                            #variant_match_bodies
                        }),*
                    }
                }
            }
        }
    }.into())
}

// Precondition: input must be valid struct
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

            impl _UriDisplay for #name {
                fn fmt(&self, f: &mut UriFormatter) -> fmt::Result {
                    _UriDisplay::fmt(&self.0, f)
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
    let idents_str = fields_named.named.iter().map(|v| v.ident.as_ref().unwrap().to_string());

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

            impl _UriDisplay for #name {
                fn fmt(&self, f: &mut UriFormatter) -> fmt::Result {
                    #(f.with_prefix(#idents_str, |mut _f| _UriDisplay::fmt(&self.#idents, &mut _f) )?;)*
                    Ok(())
                }
            }
        }
    }.into())
}

#[proc_macro_derive(_UriDisplay)]
pub fn derive_uri_display_value(input: TokenStream) -> TokenStream {
    real_derive_uri_display_value(input).unwrap_or_else(|diag| {
       diag.emit();
       TokenStream::empty()
    })
}
