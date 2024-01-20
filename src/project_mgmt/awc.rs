extern crate ffmpeg_next as ffmpeg;
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use strum::EnumIter;

use crate::{
    dat_files::dat54,
    utils::{transcoder, xml},
};

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct AwcPack {
    pub name: String,
    pub pack_type: AwcPackType,
    pub entries: Vec<AwcEntry>,
}

impl PartialOrd for AwcPack {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for AwcPack {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl AwcPack {
    pub fn add_entry(
        &mut self,
        proj_path: &PathBuf,
        entry_path: &Path,
        entry_name: &str,
    ) -> anyhow::Result<()> {
        let entry = AwcEntry::from_file(proj_path, entry_path, entry_name)?;
        self.entries.push(entry);
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct AwcEntry {
    path: PathBuf,
    pub name: String,
    pub looped: bool,
    pub headers: dat54::Header,
    // Retrieved from FFMPEG
    sample_rate: u32,
    samples: i64,
}

impl AwcEntry {
    pub fn from_file(proj_path: &PathBuf, entry_path: &Path, entry_name: &str) -> Result<AwcEntry> {
        let entry_path = entry_path.join(format!("{}.wav", entry_name));
        let ictx = ffmpeg::format::input(&entry_path)?;

        let input = ictx
            .streams()
            .best(ffmpeg::media::Type::Audio)
            .expect("Failed to find a audio stream ");
        let context = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

        let sample_rate = context.decoder().audio().unwrap().rate();

        let rel_path_res = entry_path.strip_prefix(proj_path);
        if rel_path_res.is_err() {
            return Err(anyhow!(
                "Given entry files are not stored in the project directory"
            ));
        }

        let rel_path = rel_path_res.unwrap();

        Ok(AwcEntry {
            path: rel_path.to_path_buf(),
            name: entry_name.to_string(),
            looped: false,
            headers: dat54::Header::default(),
            sample_rate,
            samples: ictx.duration().wrapping_mul(sample_rate.into()),
        })
    }

    pub fn generate_splitted_variant(&self, proj_loc: &Path) {
        let file_path = proj_loc.join(&self.path);
        let output_dir = proj_loc
            .join("output/awc/.packs/")
            .join(self.path.file_stem().unwrap().to_string_lossy().to_string());
        fs::create_dir_all(&output_dir);
        transcoder::split_stereo_to_mono(&file_path, &output_dir);
    }

    pub fn to_xml_stream(&self) -> Vec<AwcStream> {
        let mut streams = vec![];

        let left_name = format!("{}_left", self.name);

        streams.push(AwcStream {
            name: xml::InlineValue::new(&left_name),
            file_name: xml::InlineValue::new(&format!("{}.wav", &left_name)),
            chunks: xml::ItemList {
                item: vec![
                    AwcChunk::Peak,
                    AwcChunk::Data,
                    AwcChunk::Format(AwcFormatChunk::new(
                        self.samples.try_into().unwrap(),
                        self.sample_rate,
                    )),
                ],
            },
        });

        let right_name = format!("{}_right", self.name);

        streams.push(AwcStream {
            name: xml::InlineValue::new(&right_name),
            file_name: xml::InlineValue::new(&format!("{}.wav", &right_name)),
            chunks: xml::ItemList {
                item: vec![
                    AwcChunk::Peak,
                    AwcChunk::Data,
                    AwcChunk::Format(AwcFormatChunk::new(
                        self.samples.try_into().unwrap(),
                        self.sample_rate,
                    )),
                ],
            },
        });

        return streams;
    }
}

#[derive(
    Debug,
    serde::Deserialize,
    serde::Serialize,
    Default,
    PartialEq,
    EnumIter,
    Clone,
    Copy,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum AwcPackType {
    #[default]
    Simple,
    Radio,
}

impl ToString for AwcPackType {
    fn to_string(&self) -> String {
        match self {
            AwcPackType::Simple => String::from("Simple (Not-streamed)"),
            AwcPackType::Radio => String::from("Radio (Streamed)"),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase", rename = "AudioWaveContainer")]
pub struct AwcXML {
    pub version: xml::Value<u8>,
    pub chunk_indices: xml::Value<String>,
    #[serde(rename = "Streams")]
    pub streams: xml::ItemList<AwcStream>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(rename = "Item")]
pub struct AwcStream {
    name: xml::InlineValue,
    file_name: xml::InlineValue,
    #[serde(rename = "Chunks")]
    chunks: xml::ItemList<AwcChunk>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "Type", rename_all = "snake_case")]
enum AwcChunk {
    Peak,
    Data,
    Format(AwcFormatChunk),
}

// TODO: remove left-over or rewrite enum to use tagging and use these struct for ease of use
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(rename = "Item")]
struct AwcBaseChunk {
    #[serde(rename = "Type")]
    chunk_type: xml::InlineValue,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(rename = "Item")]
struct AwcFormatChunk {
    // #[serde(rename = "Type")]
    // chunk_type: xml::InlineValue,
    codec: xml::InlineValue,
    samples: xml::Value<u64>,
    sample_rate: xml::Value<u32>,
    // Mostly -161
    headroom: xml::Value<i16>,
    play_begin: xml::Value<i16>,
    play_end: xml::Value<i16>,
    loop_begin: xml::Value<u16>,
    loop_end: xml::Value<u16>,
    loop_point: xml::Value<i16>,
    peak: xml::Unk,
}

impl AwcFormatChunk {
    pub fn new(samples: u64, sample_rate: u32) -> Self {
        Self {
            // chunk_type: xml::InlineValue::new("format"),
            codec: xml::InlineValue::new("ADPCM"),
            samples: xml::Value::new(samples),
            sample_rate: xml::Value::new(sample_rate),
            headroom: xml::Value::new(-161),
            play_begin: xml::Value::new(0),
            play_end: xml::Value::new(0),
            loop_begin: xml::Value::new(0),
            loop_end: xml::Value::new(0),
            loop_point: xml::Value::new(-1),
            peak: xml::Unk::new("0"),
        }
    }
}
