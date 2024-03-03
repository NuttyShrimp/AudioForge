use crate::utils::xml;

#[derive(Debug, serde::Serialize)]
#[serde(rename = "Dat54", rename_all = "PascalCase")]
pub struct Dat54Xml {
    pub version: xml::Value<u8>,
    pub container_paths: xml::ItemList<xml::InlineValue>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
enum Dat54Item {
    SimpleSound(Dat54SimpleSound),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Dat54SimpleSound {}
