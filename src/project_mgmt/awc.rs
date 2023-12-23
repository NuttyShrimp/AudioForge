extern crate ffmpeg_next as ffmpeg;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use strum::EnumIter;

use crate::dat_files::dat54;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AwcPack {
    pub name: String,
    pub pack_type: AwcPackType,
    pub entries: Vec<AwcEntry>,
}

impl AwcPack {
    pub fn add_entry(
        &mut self,
        proj_path: &PathBuf,
        entry_path: &PathBuf,
        entry_name: &str,
    ) -> anyhow::Result<()> {
        let entry = AwcEntry::from_file(proj_path, entry_path, entry_name)?;
        self.entries.push(entry);
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AwcEntry {
    left_path: PathBuf,
    right_path: PathBuf,
    pub name: String,
    pub looped: bool,
    pub headers: dat54::Header,
    // Retrieved from FFMPEG
    sample_rate: u32,
    samples: i64,
}

impl AwcEntry {
    pub fn from_file(
        proj_path: &PathBuf,
        entry_path: &PathBuf,
        entry_name: &str,
    ) -> Result<AwcEntry> {
        let left_path = entry_path.join(format!("{}-left.wav", entry_name));
        let ictx = ffmpeg::format::input(&left_path)?;

        let input = ictx
            .streams()
            .best(ffmpeg::media::Type::Audio)
            .expect("Failed to find a audio stream ");
        let context = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

        let sample_rate = context.decoder().audio().unwrap().rate();

        let rel_path_res = proj_path.strip_prefix(entry_path);
        if rel_path_res.is_err() {
            return Err(anyhow!(
                "Given entry files are not stored in the project directory"
            ));
        }

        let rel_path = rel_path_res.unwrap();

        Ok(AwcEntry {
            left_path: rel_path.join(format!("{}-left.wav", entry_name)),
            right_path: rel_path.join(format!("{}-right.wav", entry_name)),
            name: entry_name.to_string(),
            looped: false,
            headers: dat54::Header::default(),
            sample_rate,
            samples: ictx.duration().wrapping_mul(sample_rate.into()),
        })
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Default, PartialEq, EnumIter, Clone)]
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
