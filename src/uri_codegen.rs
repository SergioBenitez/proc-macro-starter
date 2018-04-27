use syn::*;
use quote::{Tokens, ToTokens};
use ext::*;

use codegen_ext::*;
use FieldMember;

#[derive(Copy, Clone)]
pub enum FieldOrigin {
    Variant,
    Struct
}

pub struct FieldMembersNode<'f> {
    members: Vec<FieldMember<'f>>,
    origin: FieldOrigin,
    named: bool
}

impl<'f> FieldMembersNode<'f> {
    pub fn new(fields: &'f Fields, origin: FieldOrigin) -> FieldMembersNode<'f> {
        FieldMembersNode { members: fields.to_field_members(), origin: origin, named: fields.is_named() }
    }
}

fn field_member_to_variable(fm: &FieldMember, origin: FieldOrigin) -> Tokens {
    match origin {
        FieldOrigin::Struct => {
            let mem = &fm.member;
            quote!(self.#mem)
        },
        FieldOrigin::Variant => {
            fm.tokens() // TODO: change to ToTokens?
        }
    }
}

impl<'f> ToTokens for FieldMembersNode<'f> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let uri_display_calls = self.members.iter().map(|fm| {
            let var = field_member_to_variable(fm, self.origin);
            if let Some(ident) = fm.field.ident {
                let var_str = ident.as_ref();
                quote!(f.write_named_value(#var_str, &#var)?;)
            } else {
                quote!(f.write_value(&#var)?;)
            }
        });

        let concat_with_result = quote! {
            #(#uri_display_calls)*
            Ok(())
        };

        tokens.append_all(concat_with_result.into_iter());
    }
}

pub struct StructNode<'a, 'f, 'g> {
    name: &'a Ident,
    field_members: FieldMembersNode<'f>,
    lifetimes: &'g Generics
}

impl<'a, 'f, 'g> StructNode<'a, 'f, 'g> {
    pub fn new(data_struct: &'f DataStruct, name: &'a Ident, lifetimes: &'g Generics) -> StructNode<'a, 'f, 'g> {
        StructNode {
            name: name,
            field_members: FieldMembersNode::new(&data_struct.fields, FieldOrigin::Struct),
            lifetimes: lifetimes
        }
    }
}

impl<'a, 'f, 'g> ToTokens for StructNode<'a, 'f, 'g> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let field_members = &self.field_members;
        let uri_display_body = quote!(#field_members);
        let uri_display_impl = wrap_in_fmt_and_impl(uri_display_body, self.name, self.lifetimes);
        tokens.append_all(uri_display_impl.into_iter());
    }
}

pub struct VariantNode<'f> {
    name: &'f Ident,
    field_members: FieldMembersNode<'f>,
}

impl<'f> VariantNode<'f> {
    pub fn new(variant: &'f Variant) -> VariantNode<'f> {
        VariantNode {
            name: &variant.ident,
            field_members: FieldMembersNode::new(&variant.fields, FieldOrigin::Variant)
        }
    }
}

pub struct EnumNode<'a, 'f, 'g> {
    name: &'a Ident,
    variants: Vec<VariantNode<'f>>,
    lifetimes: &'g Generics
}

impl<'a, 'f, 'g>EnumNode<'a, 'f, 'g> {
    pub fn new(data_enum: &'f DataEnum, name: &'a Ident, lifetimes: &'g Generics) -> EnumNode<'a, 'f, 'g> {
        let variant_nodes : Vec<VariantNode<'f>> = data_enum.variants.iter().map(|v| VariantNode::new(v)).collect();
        EnumNode {
            name: name,
            variants: variant_nodes,
            lifetimes: lifetimes
        }
    }
}

impl<'a, 'f, 'g> ToTokens for EnumNode<'a, 'f, 'g> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let enum_name = &self.name;
        let arms = self.variants.iter().map(|v| {
            let arm_name = v.name;
            let match_refs = v.field_members.members.iter().enumerate().map(|(i, fm)| (i, fm.field)).map(field_to_match_ref);
            let refs = match v.field_members.named { // surround?
                true => quote!({#(#match_refs),*}),
                false => quote!((#(#match_refs),*))
            };
            let field_members = &v.field_members;
            let uri_display_body_for_arm = quote!(#field_members);
            quote! {
                #enum_name::#arm_name #refs => { #uri_display_body_for_arm }
            }
        });
        let uri_display_body = quote! {
            match *self {
                #(#arms),*
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
