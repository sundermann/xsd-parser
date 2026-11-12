use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct AliasesType {
    #[serde(
        skip_serializing_if = "::core::option::Option::is_none",
        rename = "Optional"
    )]
    pub optional: OptionalType,
    #[serde(
        default,
        skip_serializing_if = "::std::vec::Vec::is_empty",
        rename = "Items"
    )]
    pub items: ItemsType,
    #[serde(
        default,
        skip_serializing_if = "::std::vec::Vec::is_empty",
        rename = "BoxedItems"
    )]
    pub boxed_items: Box<ItemsType>,
    #[serde(default)]
    #[serde(with = "::serde_with::As::<::core::option::Option<[::serde_with::Same; 64]>>")]
    #[serde(
        skip_serializing_if = "::core::option::Option::is_none",
        rename = "OptionalArray"
    )]
    pub optional_array: OptionalArrayType,
}
pub type OptionalType = Option<String>;
pub type ItemsType = Vec<String>;
pub type OptionalArrayType = Option<ArrayType>;
pub type ArrayType = [String; 64usize];
