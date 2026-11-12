use std::path::Path;

use quote::ToTokens;
use xsd_parser::{
    config::{IdentTriple, OptimizerFlags, RenderStep, Schema, TypedefMode},
    exec_generator, exec_interpreter, exec_optimizer, exec_parser, exec_render,
    models::{
        data::{ComplexData, ComplexDataStruct, DataTypeVariant, StructMode},
        meta::{MetaTypeVariant, ReferenceMeta},
        schema::MaxOccurs,
    },
    Config, IdentType, TypeIdent,
};

use crate::utils::{generate_test_validate, ConfigEx};

#[test]
fn generate_option_and_vector_aliases() {
    generate_custom("tests/feature/serde_skip/expected/aliases.rs", |_| {});
}

pub(super) fn generate_custom(expected: &str, customize: impl Fn(&mut ComplexDataStruct<'_>)) {
    let mut config = Config::test_default()
        .set_optimizer_flags(OptimizerFlags::empty())
        .with_generate([(IdentType::Type, "tns:AliasesType")])
        .with_typedef_mode(TypedefMode::Typedef)
        .with_derive(["Debug", "PartialEq"])
        .with_render_step(RenderStep::TypesSerdeQuickXml {
            large_arrays: true,
            skip_none: true,
            skip_empty_vecs: true,
        });
    config.parser.schemas.push(Schema::File(
        Path::new("tests/feature/serde_skip/aliases.xsd")
            .canonicalize()
            .unwrap(),
    ));
    let schemas = exec_parser(config.parser).unwrap();
    let mut meta = exec_interpreter(config.interpreter, &schemas).unwrap();
    // Use the public meta model to exercise aliases whose outer Option or Vec
    // is hidden from a procedural macro inspecting only the Rust field syntax.
    let array = IdentTriple::from((IdentType::Type, "tns:ArrayType"))
        .resolve(&schemas)
        .unwrap();
    for (name, target, min, max) in [
        (
            "tns:OptionalType",
            TypeIdent::STRING,
            0,
            MaxOccurs::Bounded(1),
        ),
        ("tns:ItemsType", TypeIdent::STRING, 0, MaxOccurs::Unbounded),
        (
            "tns:ArrayType",
            TypeIdent::STRING,
            64,
            MaxOccurs::Bounded(64),
        ),
        ("tns:OptionalArrayType", array, 0, MaxOccurs::Bounded(1)),
    ] {
        let ident = IdentTriple::from((IdentType::Type, name))
            .resolve(&schemas)
            .unwrap();
        let mut reference = ReferenceMeta::new(target);
        reference.min_occurs = min;
        reference.max_occurs = max;
        meta.items.get_mut(&ident).unwrap().variant = MetaTypeVariant::Reference(reference);
    }
    let meta = exec_optimizer(config.optimizer, meta).unwrap();
    let mut data = exec_generator(config.generator, &schemas, &meta).unwrap();
    for item in data.items.values_mut() {
        if let DataTypeVariant::Complex(ComplexData::Struct { type_, .. }) = &mut item.variant {
            if let StructMode::Sequence { elements, .. } = &mut type_.mode {
                for element in elements {
                    if element.s_name == "BoxedItems" {
                        element.need_indirection = true;
                    }
                }
            }
            customize(type_);
        }
    }
    let module = exec_render(config.renderer, &data).unwrap();
    generate_test_validate(module.to_token_stream().to_string(), expected);
}

#[cfg(not(feature = "update-expectations"))]
mod model {
    include!("expected/aliases.rs");
}

#[test]
#[cfg(not(feature = "update-expectations"))]
fn recognizes_optional_and_vector_aliases_including_boxed_fields() {
    let empty: model::AliasesType = serde_json::from_str("{}").unwrap();
    assert!(empty.optional.is_none());
    assert!(empty.items.is_empty());
    assert!(empty.boxed_items.is_empty());
    assert!(empty.optional_array.is_none());
    assert_eq!(serde_json::to_string(&empty).unwrap(), "{}");
    let value = serde_json::json!({
        "Optional": "value", "Items": ["item"], "BoxedItems": ["boxed"],
        "OptionalArray": vec!["array"; 64],
    });
    let decoded: model::AliasesType = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), value);
}
