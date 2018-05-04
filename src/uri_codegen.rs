use ext::*;
use codegen_ext::*;
use syn::*;
use quote::{Tokens, ToTokens};
use spanned::Spanned;

fn field_member_to_variable(fm: &FieldMember) -> Tokens {
    match fm.origin {
        Origin::Struct => {
            let mem = &fm.member;
            quote!(self.#mem)
        },
        Origin::Enum => {
            fm.tokens() // TODO: change to ToTokens?
        }
    }
}

impl<'f> ToTokens for FieldMember<'f> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let var = field_member_to_variable(&self);
        let span = self.field.ty.span();
        let uri_display_call = match self.field.ident {
            Some(ident) => {
                let var_str = ident.as_ref();
                quote_spanned! { span.into() => f.write_named_value(#var_str, &#var)?; }
            },
            None => quote_spanned! { span.into() => f.write_value(&#var)?; }
        };
        tokens.append_all(uri_display_call.into_iter());
    }
}

pub struct StructNode<'i, 'f, 'g> {
    name: &'i Ident,
    fields: &'f Fields,
    lifetimes: &'g Generics
}

impl<'i, 'f, 'g> StructNode<'i, 'f, 'g> {
    pub fn new(data_struct: &'f DataStruct, name: &'i Ident, lifetimes: &'g Generics) -> StructNode<'i, 'f, 'g> {
        StructNode {
            name: name,
            fields: &data_struct.fields,
            lifetimes: lifetimes
        }
    }
}

impl<'i, 'f, 'g> ToTokens for StructNode<'i, 'f, 'g> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let field_members = self.fields.to_field_members(Origin::Struct);
        let uri_display_body = quote! { #(#field_members);* Ok(()) };
        let uri_display_impl = wrap_in_fmt_and_impl(uri_display_body, self.name, self.lifetimes);
        tokens.append_all(uri_display_impl.into_iter());
    }
}

pub struct VariantNode<'a, 'b> {
    name: &'a Ident,
    fields: &'a Fields,
    enum_name: &'b Ident
}

impl<'a, 'b> VariantNode<'a, 'b> {
    pub fn new(variant: &'a Variant, enum_name: &'b Ident) -> VariantNode<'a, 'b> {
        VariantNode {
            name: &variant.ident,
            fields: &variant.fields,
            enum_name: enum_name
        }
    }
}

impl<'a, 'b> ToTokens for VariantNode<'a, 'b> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let enum_name = self.enum_name;
        let arm_name = self.name;
        let refs = self.fields.ref_match_tokens();

        let field_members = self.fields.to_field_members(Origin::Enum);
        let uri_display_body = quote! { #(#field_members);* Ok(()) };

        let uri_display_arm = quote! {
            #enum_name::#arm_name #refs => { #uri_display_body }
        };
        tokens.append_all(uri_display_arm.into_iter());
    }
}

pub struct EnumNode<'a, 'b, 'g> {
    name: &'b Ident,
    variants: Vec<VariantNode<'a, 'b>>,
    lifetimes: &'g Generics
}

impl<'a, 'b, 'g>EnumNode<'a, 'b, 'g> {
    pub fn new(data_enum: &'a DataEnum, name: &'b Ident, lifetimes: &'g Generics) -> EnumNode<'a, 'b, 'g> {
        let variant_nodes = data_enum.variants.iter()
            .map(|v| VariantNode::new(v, name))
            .collect::<Vec<VariantNode<'a, 'b>>>();
        EnumNode {
            name: name,
            variants: variant_nodes,
            lifetimes: lifetimes
        }
    }
}

impl<'a, 'b, 'g> ToTokens for EnumNode<'a, 'b, 'g> {
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
    let (impl_generics, ty_generics, maybe_where_clause) = generics.split_for_impl();
    let scope = Ident::from(format!("scope_{}", name.to_string().to_lowercase()));
    let uri_display_bounds = generics.type_params()
        .map(|p| p.ident)
        .map(|i| quote! { #i : _UriDisplay });
    let where_uri_display_bound = match maybe_where_clause {
        Some(where_clause) if !where_clause.predicates.is_empty() => 
            quote! { #where_clause, #(#uri_display_bounds),* },
        _ => quote! { where #(#uri_display_bounds),* }
    };

    quote! {
        mod #scope {
            extern crate std;
            extern crate rocket;

            use self::std::prelude::v1::*;
            use self::std::fmt;
            use self::rocket::http::uri::*;

            impl #impl_generics _UriDisplay for #name #ty_generics #where_uri_display_bound {
                #tokens
            }
        }
    }
}
