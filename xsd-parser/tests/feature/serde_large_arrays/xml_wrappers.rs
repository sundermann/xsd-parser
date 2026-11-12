use std::path::Path;

use quote::ToTokens;
use xsd_parser::{
    config::{GeneratorFlags, IdentTriple, OptimizerFlags, RenderStep, Schema, TypedefMode},
    exec_generator, exec_interpreter, exec_optimizer, exec_parser, exec_render,
    models::{
        meta::{MetaTypeVariant, ReferenceMeta},
        schema::MaxOccurs,
    },
    Config, IdentType, TypeIdent,
};

use crate::utils::ConfigEx;

fn generate(large_arrays: bool, skip: bool) -> String {
    let mut config = Config::test_default()
        .set_optimizer_flags(OptimizerFlags::empty())
        .with_generator_flags(
            GeneratorFlags::FLATTEN_CONTENT
                | GeneratorFlags::NILLABLE_TYPE_SUPPORT
                | GeneratorFlags::MIXED_TYPE_SUPPORT,
        )
        .with_generate([
            (IdentType::Type, "tns:NillableType"),
            (IdentType::Type, "tns:MixedType"),
            (IdentType::Type, "tns:AliasesType"),
        ])
        .with_typedef_mode(TypedefMode::Typedef)
        .with_render_step(RenderStep::TypesSerdeQuickXml {
            large_arrays,
            skip_none: false,
            skip_empty_vecs: skip,
        });
    config.parser.schemas.push(Schema::File(
        Path::new("tests/feature/serde_large_arrays/xml_wrappers.xsd")
            .canonicalize()
            .unwrap(),
    ));
    let schemas = exec_parser(config.parser).unwrap();
    let mut meta = exec_interpreter(config.interpreter, &schemas).unwrap();
    for (name, min, max) in [
        ("tns:RowType", 64, MaxOccurs::Bounded(64)),
        ("tns:ItemsType", 0, MaxOccurs::Unbounded),
    ] {
        let ident = IdentTriple::from((IdentType::Type, name))
            .resolve(&schemas)
            .unwrap();
        let mut reference = ReferenceMeta::new(TypeIdent::STRING);
        reference.min_occurs = min;
        reference.max_occurs = max;
        meta.items.get_mut(&ident).unwrap().variant = MetaTypeVariant::Reference(reference);
    }
    for (name, target) in [
        ("tns:NillableRowType", "tns:RowType"),
        ("tns:NillableItemsType", "tns:ItemsType"),
    ] {
        let ident = IdentTriple::from((IdentType::Type, name))
            .resolve(&schemas)
            .unwrap();
        let target = IdentTriple::from((IdentType::Type, target))
            .resolve(&schemas)
            .unwrap();
        let mut reference = ReferenceMeta::new(target);
        reference.nillable = true;
        meta.items.get_mut(&ident).unwrap().variant = MetaTypeVariant::Reference(reference);
    }
    let meta = exec_optimizer(config.optimizer, meta).unwrap();
    let data = exec_generator(config.generator, &schemas, &meta).unwrap();
    exec_render(config.renderer, &data)
        .unwrap()
        .to_token_stream()
        .to_string()
}

#[test]
fn xml_wrappers_do_not_receive_adapters_for_their_inner_aliases() {
    let code = generate(true, false);
    assert!(code.contains("Nillable < RowType >"), "{code}");
    assert!(code.contains("Mixed < RowType >"), "{code}");
    assert_eq!(code, generate(false, false));
}

#[test]
fn xml_wrappers_are_not_treated_as_vector_fields() {
    assert_eq!(generate(true, true), generate(true, false));
}
