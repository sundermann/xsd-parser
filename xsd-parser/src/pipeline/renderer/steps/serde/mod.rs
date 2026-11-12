mod large_arrays;
mod quick_xml;
mod serde_xml_rs_v7;
mod serde_xml_rs_v8;
mod skip_empty;

use std::str::FromStr;

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;

use crate::models::{
    code::IdentPath,
    data::{ComplexDataAttribute, ComplexDataContent, ComplexDataElement, Occurs, PathData},
    meta::{AttributeMetaVariant, ElementMetaVariant, MetaTypes},
};
use crate::TypeIdent;

pub use self::quick_xml::SerdeQuickXmlTypesRenderStep;
pub use self::serde_xml_rs_v7::SerdeXmlRsV7TypesRenderStep;
pub use self::serde_xml_rs_v8::SerdeXmlRsV8TypesRenderStep;

use super::super::Context;

impl PathData {
    fn serde_target_ident<'a>(&self, ident: Option<&'a TypeIdent>) -> Option<&'a TypeIdent> {
        // Generic XML wrappers change the value being serialized. The schema
        // identifier describes their inner value, not the wrapper itself.
        ident.filter(|_| self.generics.is_empty())
    }
}

impl ComplexDataElement<'_> {
    fn serde_target_ident(&self) -> Option<&TypeIdent> {
        let ident = match &self.meta().variant {
            ElementMetaVariant::Type { type_, .. } => Some(type_),
            _ => None,
        };
        self.target_type.serde_target_ident(ident)
    }
}

impl ComplexDataContent<'_> {
    fn is_empty_string_content(&self, types: &MetaTypes) -> bool {
        if let Some(ident) = &self.simple_type {
            if let Some(ident) = types.get_resolved_ident(ident) {
                if ident.matches(&TypeIdent::STRING) {
                    return true;
                }
            }
        }

        false
    }
}

fn get_derive<I>(ctx: &Context<'_, '_>, extra: I) -> TokenStream
where
    I: IntoIterator<Item = &'static str>,
{
    ctx.add_usings(["serde::Serialize", "serde::Deserialize"]);

    super::get_derive(ctx, DERIVE.into_iter().chain(extra))
}

fn get_dyn_type_traits(ctx: &Context<'_, '_>) -> TokenStream {
    super::get_dyn_type_traits(
        ctx,
        [
            &IdentPath::from_str("serde::Serialize").unwrap(),
            &IdentPath::from_str("serde::de::DeserializeOwned").unwrap(),
        ],
    )
}

const DERIVE: [&str; 2] = ["Serialize", "Deserialize"];

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)] // Independent renderer policies can be combined.
pub(crate) struct SerdeOptions {
    large_arrays: bool,
    arrays_as_vec: bool,
    skip_none: bool,
    skip_empty_vecs: bool,
}

impl Occurs {
    fn array_to_vec(self) -> Self {
        match self {
            Self::StaticList(_) => Self::DynamicList,
            x => x,
        }
    }
}

#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)] // Mirrors independent Serde field attributes.
pub(crate) struct FieldOverrides {
    serialize: bool,
    deserialize: bool,
    has_default: bool,
    has_custom_default: bool,
    skip: bool,
}

fn is_assignment(token: Option<&TokenTree>) -> bool {
    matches!(token, Some(TokenTree::Punct(p)) if p.as_char() == '=')
}

fn field_overrides(attributes: &[TokenStream]) -> FieldOverrides {
    let mut result = FieldOverrides::default();
    for attribute in attributes {
        let mut tokens = attribute.clone().into_iter();
        let Some(TokenTree::Ident(name)) = tokens.next() else {
            continue;
        };
        if name == "serde_as" {
            result.serialize = true;
            result.deserialize = true;
            continue;
        }
        if name != "serde" {
            continue;
        }
        let Some(TokenTree::Group(arguments)) = tokens.next() else {
            continue;
        };
        let mut arguments = arguments.stream().into_iter().peekable();
        while let Some(token) = arguments.next() {
            if let TokenTree::Ident(name) = token {
                match name.to_string().as_str() {
                    "with" => {
                        result.serialize = true;
                        result.deserialize = true;
                    }
                    "serialize_with" => result.serialize = true,
                    "deserialize_with" => result.deserialize = true,
                    "default" => {
                        result.has_default = true;
                        result.has_custom_default |= is_assignment(arguments.peek());
                    }
                    "skip" | "skip_serializing" | "skip_serializing_if" => result.skip = true,
                    _ => (),
                }
            }
        }
    }
    result
}

impl ComplexDataElement<'_> {
    fn serde_field_attributes(
        &self,
        ctx: &Context<'_, '_>,
        options: SerdeOptions,
        overrides: &FieldOverrides,
        arguments: &TokenStream,
    ) -> TokenStream {
        let array_attribute = self.serde_array_field_attribute(ctx, options, overrides);
        let skip = skip_empty::arguments(
            ctx,
            self.occurs,
            self.serde_target_ident(),
            self.meta().min_occurs == 0,
            overrides,
            options,
        );
        quote!(#array_attribute #[serde(#skip #arguments)])
    }
}

impl ComplexDataContent<'_> {
    fn serde_field_attributes(
        &self,
        ctx: &Context<'_, '_>,
        options: SerdeOptions,
        overrides: &FieldOverrides,
        arguments: &TokenStream,
    ) -> TokenStream {
        let array_attribute = self.serde_array_attribute(ctx, options, overrides);
        let skip = skip_empty::arguments(
            ctx,
            self.occurs,
            self.target_type.serde_target_ident(self.simple_type),
            self.min_occurs == 0,
            overrides,
            options,
        );
        quote!(#array_attribute #[serde(#skip #arguments)])
    }
}

impl ComplexDataAttribute<'_> {
    fn serde_field_attributes(
        &self,
        ctx: &Context<'_, '_>,
        options: SerdeOptions,
        overrides: &FieldOverrides,
        arguments: &TokenStream,
    ) -> TokenStream {
        let array_attribute = self.serde_array_attribute(ctx, options, overrides);
        let target = match &self.meta.variant {
            AttributeMetaVariant::Type(type_) => Some(type_),
            _ => None,
        };
        let skip = skip_empty::arguments(
            ctx,
            if self.is_option {
                Occurs::Optional
            } else {
                Occurs::Single
            },
            self.target_type.serde_target_ident(target),
            self.is_option || self.default_value.is_some(),
            overrides,
            options,
        );
        quote!(#array_attribute #[serde(#skip #arguments)])
    }
}
