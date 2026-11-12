use std::path::Path;

use quote::{quote, ToTokens};
use xsd_parser::{
    config::{RenderStep, Schema, SerdeXmlRsVersion, TypedefMode},
    exec_generator, exec_interpreter, exec_optimizer, exec_parser, exec_render,
    models::data::{ComplexData, DataTypeVariant},
    Config, IdentType,
};

use crate::utils::{generate_test_validate, ConfigEx};

fn config() -> Config {
    Config::test_default()
        .with_generate([(IdentType::Type, "tns:AttributesType")])
        .with_typedef_mode(TypedefMode::Typedef)
        .with_derive(["Debug", "PartialEq"])
}

fn generate(expected: &str, mut config: Config) {
    config.parser.schemas.push(Schema::File(
        Path::new("tests/feature/serde_skip/schema_defaults.xsd")
            .canonicalize()
            .unwrap(),
    ));
    let schemas = exec_parser(config.parser).unwrap();
    let meta = exec_interpreter(config.interpreter, &schemas).unwrap();
    let meta = exec_optimizer(config.optimizer, meta).unwrap();
    let mut data = exec_generator(config.generator, &schemas, &meta).unwrap();
    for item in data.items.values_mut() {
        if let DataTypeVariant::Complex(ComplexData::Struct { type_, .. }) = &mut item.variant {
            for attribute in &mut type_.attributes {
                // Nonempty list defaults currently need a custom value renderer;
                // the generic lexical conversion only supports empty defaults.
                attribute.default_value = Some(Box::new(quote!(vec![
                    String::from("fallback"),
                    String::from("other"),
                ])));
            }
        }
    }
    let module = exec_render(config.renderer, &data).unwrap();
    generate_test_validate(module.to_token_stream().to_string(), expected);
}

#[test]
fn generate_quick_xml() {
    generate(
        "tests/feature/serde_skip/expected/schema_default_quick_xml.rs",
        config()
            .with_serde_quick_xml()
            .with_render_step(RenderStep::TypesSerdeQuickXml {
                large_arrays: false,
                skip_none: false,
                skip_empty_vecs: true,
            }),
    );
}

#[test]
fn generate_xml_rs_v7() {
    generate(
        "tests/feature/serde_skip/expected/schema_default_xml_rs_v7.rs",
        config()
            .with_serde_xml_rs(SerdeXmlRsVersion::Version07AndBelow)
            .with_render_step(RenderStep::TypesSerdeXmlRs {
                version: SerdeXmlRsVersion::Version07AndBelow,
                large_arrays: false,
                skip_none: false,
                skip_empty_vecs: true,
            }),
    );
}

#[test]
fn generate_xml_rs_v8() {
    generate(
        "tests/feature/serde_skip/expected/schema_default_xml_rs_v8.rs",
        config()
            .with_serde_xml_rs(SerdeXmlRsVersion::Version08AndAbove)
            .with_render_step(RenderStep::TypesSerdeXmlRs {
                version: SerdeXmlRsVersion::Version08AndAbove,
                large_arrays: false,
                skip_none: false,
                skip_empty_vecs: true,
            }),
    );
}

#[cfg(not(feature = "update-expectations"))]
mod runtime {
    mod quick_xml {
        include!("expected/schema_default_quick_xml.rs");
    }
    mod xml_rs_v7 {
        include!("expected/schema_default_xml_rs_v7.rs");
    }
    mod xml_rs_v8 {
        include!("expected/schema_default_xml_rs_v8.rs");
    }

    #[test]
    fn empty_list_attribute_with_schema_default_round_trips() {
        let value = serde_json::json!({"@values": []});
        let encoded = super::super::round_trip_json::<quick_xml::AttributesType>(value.clone());
        assert_eq!(encoded, value);
        let encoded = super::super::round_trip_json::<xml_rs_v8::AttributesType>(value.clone());
        assert_eq!(encoded, value);
        let value = serde_json::json!({"values": []});
        let encoded = super::super::round_trip_json::<xml_rs_v7::AttributesType>(value.clone());
        assert_eq!(encoded, value);

        macro_rules! check_xml {
            ($module:ident, $serialize:path, $deserialize:path) => {{
                let original = $module::AttributesType { values: vec![] };
                let xml = $serialize(&original).unwrap();
                assert!(xml.contains("values=\"\""), "{xml}");
                let decoded: $module::AttributesType = $deserialize(&xml).unwrap();
                assert_eq!(decoded, original);
            }};
        }
        check_xml!(
            quick_xml,
            ::quick_xml::se::to_string,
            ::quick_xml::de::from_str
        );
        check_xml!(
            xml_rs_v8,
            ::serde_xml_rs::to_string,
            ::serde_xml_rs::from_str
        );
    }
}
