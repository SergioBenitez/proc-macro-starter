use syn::*;
use quote::{Tokens, ToTokens};
use ext::*;

use codegen_ext::*;
use FieldMember;

fn field_member_to_variable(fm: &FieldMember) -> Tokens {
    match fm.origin {
        FieldOrigin::Struct => {
            let mem = &fm.member;
            quote!(self.#mem)
        },
        FieldOrigin::Enum => {
            fm.tokens() // TODO: change to ToTokens?
        }
    }
}

impl<'f> ToTokens for FieldMember<'f> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let var = field_member_to_variable(&self);
        let uri_display_call = match self.field.ident {
            Some(ident) => {
                let var_str = ident.as_ref();
                quote!(f.write_named_value(#var_str, &#var)?;)
            },
            None => quote!(f.write_value(&#var)?;)
        };
        tokens.append_all(uri_display_call.into_iter());
    }
}

pub struct StructNode<'a, 'f, 'g> {
    name: &'a Ident,
    fields: &'f Fields,
    lifetimes: &'g Generics
}

impl<'a, 'f, 'g> StructNode<'a, 'f, 'g> {
    pub fn new(data_struct: &'f DataStruct, name: &'a Ident, lifetimes: &'g Generics) -> StructNode<'a, 'f, 'g> {
        let fields = &data_struct.fields;
        StructNode {
            name: name,
            fields: fields,
            lifetimes: lifetimes
        }
    }
}

impl<'a, 'f, 'g> ToTokens for StructNode<'a, 'f, 'g> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let field_members = self.fields.to_field_members(FieldOrigin::Struct);
        let uri_display_body = quote! { #(#field_members);* Ok(()) };
        let uri_display_impl = wrap_in_fmt_and_impl(uri_display_body, self.name, self.lifetimes);
        tokens.append_all(uri_display_impl.into_iter());
    }
}

pub struct VariantNode<'f, 'a> {
    name: &'f Ident,
    fields: &'f Fields,
    enum_name: &'a Ident
}

impl<'f, 'a> VariantNode<'f, 'a> {
    pub fn new(variant: &'f Variant, enum_name: &'a Ident) -> VariantNode<'f, 'a> {
        // let field_members = variant.fields.to_field_members(FieldOrigin::Enum);
        VariantNode {
            name: &variant.ident,
            fields: &variant.fields,
            enum_name: enum_name
        }
    }
}

impl<'f, 'a> ToTokens for VariantNode<'f, 'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let enum_name = self.enum_name;
        let arm_name = self.name;
        let refs = self.fields.ref_match_tokens();
        let field_members = self.fields.to_field_members(FieldOrigin::Enum);
        let uri_display_body = quote! { #(#field_members);* Ok(()) };
        let uri_display_arm = quote! {
            #enum_name::#arm_name #refs => { #uri_display_body }
        };
        tokens.append_all(uri_display_arm.into_iter());
    }
}

pub struct EnumNode<'a, 'f, 'g> {
    name: &'a Ident,
    variants: Vec<VariantNode<'f, 'a>>,
    lifetimes: &'g Generics
}

impl<'a, 'f, 'g>EnumNode<'a, 'f, 'g> {
    pub fn new(data_enum: &'f DataEnum, name: &'a Ident, lifetimes: &'g Generics) -> EnumNode<'a, 'f, 'g> {
        let variant_nodes = data_enum.variants.iter()
            .map(|v| VariantNode::new(v, name))
            .collect::<Vec<VariantNode<'f, 'a>>>();
        EnumNode {
            name: name,
            variants: variant_nodes,
            lifetimes: lifetimes
        }
    }
}

impl<'a, 'f, 'g> ToTokens for EnumNode<'a, 'f, 'g> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let variants = &self.variants;
        let uri_display_body = quote! {
            match *self {
                #(#variants),*
            }
        };
        let uri_display_impl = wrap_in_fmt_and_impl(uri_display_body, self.name, self.lifetimes);
        tokens.append_all(uri_display_impl.into_iter());
    }
}

fn wrap_in_fmt_and_impl(tokens: Tokens, name: &Ident, generics: &Generics) -> Tokens {
    wrap_in_impl(wrap_in_fmt(tokens), name, generics)
}

fn wrap_in_fmt(tokens: Tokens) -> Tokens {
    quote! {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            #tokens
        }
    }
}

fn wrap_in_impl(tokens: Tokens, name: &Ident, generics: &Generics) -> Tokens {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
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
