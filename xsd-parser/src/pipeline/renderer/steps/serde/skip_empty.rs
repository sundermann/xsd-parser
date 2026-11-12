use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    config::TypedefMode,
    models::{
        data::{DataTypeVariant, Occurs},
        TypeIdent,
    },
    pipeline::renderer::Context,
};

use super::{FieldOverrides, SerdeOptions};

pub(super) fn arguments<'a>(
    ctx: &'a Context<'_, '_>,
    mut occurs: Occurs,
    mut target: Option<&'a TypeIdent>,
    mut can_be_missing: bool,
    overrides: &FieldOverrides,
    options: SerdeOptions,
) -> Option<TokenStream> {
    if !options.skip_none && !options.skip_empty_vecs {
        return None;
    }
    // Retain values when omission could invoke a different default or leave a
    // custom deserializer without the input it requires.
    if overrides.skip
        || overrides.has_custom_default
        || (overrides.deserialize && !overrides.has_default)
    {
        return None;
    }
    if options.arrays_as_vec {
        occurs = occurs.array_to_vec();
    }
    let mut visited = HashSet::new();
    while occurs == Occurs::Single {
        let ident = target?;
        if !visited.insert(ident) {
            return None;
        }
        let item = ctx.meta.types.items.get(ident)?;
        let DataTypeVariant::Reference(reference) = &item.variant else {
            return None;
        };
        if reference.mode != TypedefMode::Typedef {
            return None;
        }
        occurs = reference.occurs;
        if options.arrays_as_vec {
            occurs = occurs.array_to_vec();
        }
        can_be_missing |= reference.meta.min_occurs == 0;
        target = reference
            .target_type
            .serde_target_ident(Some(&reference.meta.type_));
    }
    let (predicate, default) = match occurs {
        Occurs::Optional if options.skip_none => {
            // Native Option deserialization handles missing fields through aliases.
            // Adapters supply their own defaults when they need one.
            ("::core::option::Option::is_none".into(), false)
        }
        Occurs::DynamicList if options.skip_empty_vecs => (
            format!("::{}::vec::Vec::is_empty", ctx.alloc_crate),
            can_be_missing,
        ),
        _ => return None,
    };
    let default = (default && !overrides.has_default).then(|| quote!(default,));
    Some(quote!(#default skip_serializing_if = #predicate,))
}
