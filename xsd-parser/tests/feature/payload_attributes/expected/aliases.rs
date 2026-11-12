use core::ops::Deref;
use xsd_parser_types::quick_xml::ValidateError;
pub type Value = u32;
#[derive(Debug, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub struct CountType(
    ///Payload field.
    #[serde(with = "crate::feature::payload_attributes::encoded_u32")]
    pub u32,
);
impl CountType {
    pub fn new(inner: u32) -> Result<Self, ValidateError> {
        Self::validate_value(&inner)?;
        Ok(Self(inner))
    }
    #[must_use]
    pub fn into_inner(self) -> u32 {
        self.0
    }
    pub fn validate_value(value: &u32) -> Result<(), ValidateError> {
        if *value < 1u32 {
            return Err(ValidateError::LessThan("1"));
        }
        Ok(())
    }
}
impl From<CountType> for u32 {
    fn from(value: CountType) -> u32 {
        value.0
    }
}
impl TryFrom<u32> for CountType {
    type Error = ValidateError;
    fn try_from(value: u32) -> Result<Self, ValidateError> {
        Self::new(value)
    }
}
impl Deref for CountType {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub type CountsType = Vec<UnsignedIntType>;
pub type UnsignedIntType = u32;
#[derive(Debug, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum NumberOrTextType {
    ///Enum variant.
    U32(
        ///Payload field.
        #[serde(with = "crate::feature::payload_attributes::encoded_u32")]
        u32,
    ),
    ///Enum variant.
    String(
        ///Payload field.
        String,
    ),
}
#[derive(Debug, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum CodeType {
    ///Enum variant.
    _0,
}
#[derive(Debug, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum OpenCodeType {
    ///Enum variant.
    _0,
    ///Enum variant.
    U32(
        ///Payload field.
        #[serde(with = "crate::feature::payload_attributes::encoded_u32")]
        u32,
    ),
}
#[derive(Debug, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub struct ChoiceType {
    pub content: ChoiceTypeContent,
}
#[derive(Debug, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ChoiceTypeContent {
    ///Enum variant.
    Number(
        ///Payload field.
        #[serde(with = "crate::feature::payload_attributes::encoded_u32")]
        u32,
    ),
    ///Enum variant.
    Numbers(
        ///Payload field.
        [u32; 3usize],
    ),
    ///Enum variant.
    Text(
        ///Payload field.
        String,
    ),
}
#[derive(Debug, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub struct RecordType {
    pub number: u32,
}
