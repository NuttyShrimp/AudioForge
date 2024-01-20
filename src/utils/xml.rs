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
