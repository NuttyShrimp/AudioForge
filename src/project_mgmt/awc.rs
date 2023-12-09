use crate::dat_files::dat54;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AwcPack {
    pub name: String,
    pub entries: Vec<AwcEntry>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AwcEntry {
    path: String,
    name: String,
    looped: bool,
    headers: dat54::Header,
    // Retrieved from FFMPEG
    sample_rate: f64,
    samples: f64,
}
