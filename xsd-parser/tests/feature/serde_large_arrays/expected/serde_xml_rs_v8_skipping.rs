use serde::{Deserialize, Serialize};
pub type Packet = PacketType;
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct PacketType {
    #[serde(
        skip_serializing_if = "::std::vec::Vec::is_empty",
        default,
        rename = "tns:Sample"
    )]
    pub sample: Vec<String>,
    #[serde(
        skip_serializing_if = "::std::vec::Vec::is_empty",
        default,
        rename = "tns:Small"
    )]
    pub small: Vec<u32>,
    #[serde(
        skip_serializing_if = "::std::vec::Vec::is_empty",
        default,
        rename = "tns:Threshold"
    )]
    pub threshold: Vec<u32>,
    #[serde(rename = "tns:Choice")]
    pub choice: ChoiceType,
}
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum ChoiceType {
    #[serde(rename = "tns:Number")]
    Number(Vec<u32>),
    #[serde(rename = "tns:Text")]
    Text(String),
}
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct NamedArrayType {
    #[serde(
        skip_serializing_if = "::std::vec::Vec::is_empty",
        default,
        rename = "tns:Sample"
    )]
    pub sample: Vec<String>,
    #[serde(
        skip_serializing_if = "::std::vec::Vec::is_empty",
        default,
        rename = "tns:Threshold"
    )]
    pub threshold: Vec<u32>,
}
