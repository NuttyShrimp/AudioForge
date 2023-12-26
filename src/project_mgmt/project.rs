use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use super::awc::{self, AwcPack};
use anyhow::Result;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Project {
    #[serde(skip_serializing, skip_deserializing)]
    pub location: PathBuf,
    pub awc_info: Vec<awc::AwcPack>,
}

impl Project {
    fn create_project(path: &PathBuf) -> Result<()> {
        let proj = Project {
            location: path.clone(),
            awc_info: vec![],
        };
        proj.save()?;
        Ok(())
    }

    fn open_project(path: &PathBuf) -> Result<Project> {
        let mut f = File::open(path.join("info.json"))?;
        let mut buffer = String::new();

        f.read_to_string(&mut buffer)?;
        let mut proj: Project = serde_json::from_str(&buffer)?;
        proj.location = path.clone();

        Ok(proj)
    }

    fn is_folder_a_project(path: &Path) -> bool {
        path.join("info.json").exists()
    }

    // Opens directory picker
    pub fn choose_project() -> Result<Project> {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            if !Project::is_folder_a_project(path.as_path()) {
                Project::create_project(&path)?;
            }
            let proj = Project::open_project(&path)?;
            return Ok(proj);
        }
        Err(anyhow::format_err!(""))
    }

    pub fn get_mut_entries_slice(&mut self) -> &mut [AwcPack] {
        return self.awc_info.as_mut_slice();
    }

    pub fn save(&self) -> Result<()> {
        let json_str = serde_json::to_string(self)?;
        let mut f = File::create(self.location.join("info.json"))?;
        f.write_all(json_str.as_bytes())?;
        Ok(())
    }

    pub fn add_awc_pack(&mut self, pack: AwcPack) {
        self.awc_info.push(pack);
        self.awc_info.sort();
    }
}
