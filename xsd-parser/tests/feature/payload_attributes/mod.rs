use std::{collections::BTreeSet, path::Path};

use quote::{quote, ToTokens};
use xsd_parser::{
    config::{OptimizerFlags, Schema, TypedefMode},
    exec_generator, exec_interpreter, exec_optimizer, exec_parser, exec_render,
    models::data::{ComplexData, DataTypeVariant, DataTypes, Occurs, StructMode},
    Config, IdentType,
};

use crate::utils::{generate_test_validate, ConfigEx};

fn config() -> Config {
    Config::test_default()
        .set_optimizer_flags(OptimizerFlags::RESOLVE_TYPEDEFS | OptimizerFlags::MERGE_ENUM_UNIONS)
        .with_generate([
            (IdentType::Element, "tns:Value"),
            (IdentType::Type, "tns:Count"),
            (IdentType::Type, "tns:Counts"),
            (IdentType::Type, "tns:NumberOrText"),
            (IdentType::Type, "tns:Code"),
            (IdentType::Type, "tns:OpenCode"),
            (IdentType::Type, "tns:Choice"),
            (IdentType::Type, "tns:Record"),
        ])
        .with_typedef_mode(TypedefMode::NewType)
        .with_derive([
            "Debug",
            "PartialEq",
            "serde::Serialize",
            "serde::Deserialize",
        ])
}

fn generate(input: &str, expected: &str, mut config: Config) -> BTreeSet<&'static str> {
    config
        .parser
        .schemas
        .push(Schema::File(Path::new(input).canonicalize().unwrap()));
    let schemas = exec_parser(config.parser).unwrap();
    let meta = exec_interpreter(config.interpreter, &schemas).unwrap();
    let meta = exec_optimizer(config.optimizer, meta).unwrap();
    let mut data = exec_generator(config.generator, &schemas, &meta).unwrap();
    let seen = add_attributes(&mut data);
    let module = exec_render(config.renderer, &data).unwrap();
    generate_test_validate(module.to_token_stream().to_string(), expected);
    seen
}

fn add_attributes(data: &mut DataTypes<'_>) -> BTreeSet<&'static str> {
    let mut seen = BTreeSet::new();
    let payload = quote!(doc = "Payload field.");
    let variant = quote!(doc = "Enum variant.");
    let codec = quote!(serde(
        with = "crate::feature::payload_attributes::encoded_u32"
    ));
    for item in data.items.values_mut() {
        match &mut item.variant {
            DataTypeVariant::Simple(simple) => {
                simple.payload_attributes.push(payload.clone());
                if simple.occurs == Occurs::Single {
                    simple.payload_attributes.push(codec.clone());
                }
                seen.insert("simple");
            }
            DataTypeVariant::Reference(reference) => {
                reference.payload_attributes.push(payload.clone());
                if reference.target_type.path.ident() == "u32" {
                    reference.payload_attributes.push(codec.clone());
                }
                seen.insert("reference");
            }
            DataTypeVariant::Dynamic(dynamic) => {
                dynamic.payload_attributes.push(payload.clone());
                seen.insert("dynamic");
            }
            DataTypeVariant::Enumeration(enumeration) => {
                for value in &mut enumeration.variants {
                    value.extra_attributes.push(variant.clone());
                    value.payload_attributes.push(payload.clone());
                    value.payload_attributes.push(codec.clone());
                    if value.target_type.is_some() {
                        seen.insert("enumeration payload");
                    } else {
                        seen.insert("unit variant");
                    }
                }
            }
            DataTypeVariant::Union(union) => {
                for value in &mut union.variants {
                    value.extra_attributes.push(variant.clone());
                    value.payload_attributes.push(payload.clone());
                    if value.target_type.path.ident() == "u32" {
                        value.payload_attributes.push(codec.clone());
                    }
                }
                seen.insert("union");
            }
            DataTypeVariant::Complex(complex) => {
                let mut next = Some(complex);
                while let Some(complex) = next {
                    next = match complex {
                        ComplexData::Enum {
                            type_,
                            content_type,
                        } => {
                            for value in &mut type_.elements {
                                value.extra_attributes.push(variant.clone());
                                value.payload_attributes.push(payload.clone());
                                if value.s_name == "Number" {
                                    value.payload_attributes.push(codec.clone());
                                }
                            }
                            seen.insert("choice");
                            content_type.as_deref_mut()
                        }
                        ComplexData::Struct {
                            type_,
                            content_type,
                        } => {
                            if let StructMode::All { elements, .. }
                            | StructMode::Sequence { elements, .. } = &mut type_.mode
                            {
                                for value in elements {
                                    // Payload attributes must not leak onto named fields.
                                    value
                                        .payload_attributes
                                        .push(quote!(doc = "Ignored payload."));
                                }
                            }
                            content_type.as_deref_mut()
                        }
                    };
                }
            }
            _ => (),
        }
    }
    seen
}

#[test]
fn generate_default() {
    let seen = generate(
        "tests/feature/payload_attributes/schema.xsd",
        "tests/feature/payload_attributes/expected/default.rs",
        config(),
    );
    assert_eq!(
        seen,
        BTreeSet::from([
            "simple",
            "reference",
            "enumeration payload",
            "unit variant",
            "union",
            "choice",
        ])
    );
}

#[test]
fn generate_aliases() {
    generate(
        "tests/feature/payload_attributes/schema.xsd",
        "tests/feature/payload_attributes/expected/aliases.rs",
        config().with_typedef_mode(TypedefMode::Typedef),
    );
}

#[test]
fn generate_dynamic() {
    let seen = generate(
        "tests/feature/dynamic_type/schema.xsd",
        "tests/feature/payload_attributes/expected/dynamic.rs",
        Config::test_default()
            .with_dynamic_type((IdentType::Type, "tns:animal"))
            .with_generate([(IdentType::Element, "tns:list")]),
    );
    assert!(seen.contains("dynamic"));
}

#[cfg(not(feature = "update-expectations"))]
mod default {
    include!("expected/default.rs");
}

#[cfg(not(feature = "update-expectations"))]
mod aliases {
    include!("expected/aliases.rs");
}

#[cfg(not(feature = "update-expectations"))]
mod dynamic {
    #![allow(unused_imports)]

    include!("expected/dynamic.rs");
}

#[cfg(not(feature = "update-expectations"))]
mod encoded_u32 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &u32, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("encoded-{value}"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
        let value = String::deserialize(deserializer)?;
        value
            .strip_prefix("encoded-")
            .ok_or_else(|| serde::de::Error::custom("missing prefix"))?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[test]
#[cfg(not(feature = "update-expectations"))]
fn serde_uses_payload_codec() {
    fn round_trip<T>(value: T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let xml = quick_xml::se::to_string(&value).unwrap();
        assert!(xml.contains("encoded-7"), "{xml}");
        let actual: T = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(actual, value);
    }

    round_trip(default::Value(7));
    round_trip(default::CountType(7));
    round_trip(default::ChoiceTypeContent::Number(7));
    round_trip(default::NumberOrTextType::U32(7));
    round_trip(default::OpenCodeType::U32(7));
}
