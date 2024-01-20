use anyhow::Result;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Value<T: Serialize> {
    #[serde(rename = "@value")]
    value: T,
}

impl<T: Serialize> Value<T> {
    pub fn new(val: T) -> Self {
        Self { value: val }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InlineValue {
    #[serde(rename = "$value")]
    value: String,
}

impl InlineValue {
    pub fn new(val: &str) -> Self {
        Self {
            value: val.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Unk {
    #[serde(rename = "@unk")]
    unk: String,
}

impl Unk {
    pub fn new(val: &str) -> Self {
        Self {
            unk: val.to_string(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(rename = "List")]
pub struct ItemList<T> {
    pub item: Vec<T>,
}

pub fn serialize_str<T>(xml_struct: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    // Parse to string and write to file
    let serialized = quick_xml::se::to_string(&xml_struct);

    if serialized.is_err() {
        error!("Failed to serialize xml: {:?}", serialized.unwrap_err());
        return Err(anyhow::format_err!("Failed to serialize xml"));
    }

    let serialized = serialized.unwrap();
    let serialized = r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string() + &serialized;
    Ok(serialized)
}
