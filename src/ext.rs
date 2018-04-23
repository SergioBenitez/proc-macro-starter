use std::fmt;
use std::fmt::*;
use syn::*;
use spanned::Spanned;

#[derive(Debug, Clone)]
pub struct FieldMember<'f> {
    pub field: &'f Field,
    pub member: Member
}

impl<'f> Display for FieldMember<'f> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.member {
            Member::Named(ref ident) => Display::fmt(ident, formatter),
            Member::Unnamed(ref index) => Display::fmt(&index.index, formatter)
        }
    }
}

pub trait MemberExt {
    fn named(&self) -> Option<&Ident>;
    fn unnamed(&self) -> Option<&Index>;
}

impl MemberExt for Member {
    fn named(&self) -> Option<&Ident> {
        match *self {
            Member::Named(ref named) => Some(named),
            _ => None
        }
    }

    fn unnamed(&self) -> Option<&Index> {
        match *self {
            Member::Unnamed(ref unnamed) => Some(unnamed),
            _ => None
        }
    }
}

pub(crate) trait FieldExt {
    fn to_field_member(&self, i: usize) -> FieldMember;
}

impl FieldExt for Field {
    fn to_field_member(&self, i: usize) -> FieldMember {
        if let Some(ident) = self.ident {
            FieldMember { field: &self, member: Member::Named(ident) }
        } else {
            let index = Index { index: i as u32, span: self.span().into() };
            let member = Member::Unnamed(index);
            FieldMember { field: &self, member: member }
        }
    }
}

pub(crate) trait FieldsExt {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn named(&self) -> Option<&FieldsNamed>;
    fn is_named(&self) -> bool;
    fn unnamed(&self) -> Option<&FieldsUnnamed>;
    fn is_unnamed(&self) -> bool;
    fn is_unit(&self) -> bool;
    fn nth(&self, i: usize) -> Option<&Field>;
    fn find_member(&self, member: &Member) -> Option<&Field>;
    fn to_field_members<'f>(&'f self) -> Vec<FieldMember<'f>>;
}

impl FieldsExt for Fields {
    fn len(&self) -> usize {
        match *self {
            Fields::Named(ref fields) => fields.named.len(),
            Fields::Unnamed(ref fields) => fields.unnamed.len(),
            Fields::Unit => 0
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn named(&self) -> Option<&FieldsNamed> {
        match *self {
            Fields::Named(ref named) => Some(named),
            _ => None
        }
    }

    fn is_named(&self) -> bool {
        self.named().is_some()
    }

    fn unnamed(&self) -> Option<&FieldsUnnamed> {
        match *self {
            Fields::Unnamed(ref unnamed) => Some(unnamed),
            _ => None
        }
    }

    fn is_unnamed(&self) -> bool {
        self.unnamed().is_some()
    }

    fn is_unit(&self) -> bool {
        match *self {
            Fields::Unit => true,
            _ => false
        }
    }
    
    fn to_field_members<'f>(&'f self) -> Vec<FieldMember<'f>> {
        self.iter().enumerate().map(|(index, field)| field.to_field_member(index)).collect()
    }

    fn nth(&self, i: usize) -> Option<&Field> {
        match *self {
            Fields::Named(ref fields) => fields.named.iter().nth(i),
            Fields::Unnamed(ref fields) => fields.unnamed.iter().nth(i),
            Fields::Unit => None
        }
    }

    fn find_member(&self, member: &Member) -> Option<&Field> {
        if let (Some(fields), Some(ident)) = (self.named(), member.named()) {
            fields.named.iter().find(|f| f.ident.as_ref().unwrap() == ident)
        } else if let (Some(fields), Some(member)) = (self.unnamed(), member.unnamed()) {
            fields.unnamed.iter().nth(member.index as usize)
        } else {
            None
        }
    }
}

pub trait PathExt {
    fn is(&self, global: bool, segments: &[&str]) -> bool;
    fn is_local(&self, segments: &[&str]) -> bool;
    fn is_global(&self, segments: &[&str]) -> bool;
}

impl PathExt for Path {
    fn is(&self, global: bool, segments: &[&str]) -> bool {
        if self.global() != global || self.segments.len() != segments.len() {
            return false;
        }

        for (segment, wanted) in self.segments.iter().zip(segments.iter()) {
            if segment.ident != wanted {
                return false;
            }
        }

        true
    }

    fn is_local(&self, segments: &[&str]) -> bool {
        self.is(false, segments)
    }

    fn is_global(&self, segments: &[&str]) -> bool {
        self.is(true, segments)
    }
}

pub trait DataExt {
    fn into_enum(self) -> Option<DataEnum>;
    fn into_struct(self) -> Option<DataStruct>;
    fn into_union(self) -> Option<DataUnion>;
}

impl DataExt for Data {
    fn into_enum(self) -> Option<DataEnum> {
        match self {
            Data::Enum(e) => Some(e),
            _ => None
        }
    }

    fn into_struct(self) -> Option<DataStruct> {
        match self {
            Data::Struct(s) => Some(s),
            _ => None
        }
    }

    fn into_union(self) -> Option<DataUnion> {
        match self {
            Data::Union(u) => Some(u),
            _ => None
        }
    }
}
