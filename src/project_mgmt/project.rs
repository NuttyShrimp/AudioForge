use log::{error, info};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::utils::xml;

use super::awc::{self, AwcPack, AwcXML};
use anyhow::Result;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Project {
    #[serde(skip_serializing, skip_deserializing)]
    pub location: PathBuf,
    pub awc_info: Vec<awc::AwcPack>,
}

impl Project {
    fn create_project(path: &Path) -> Result<()> {
        let proj = Project {
            location: path.to_path_buf().clone(),
            awc_info: vec![],
        };
        proj.save()?;
        Ok(())
    }

    pub fn open_project(path: &Path) -> Result<Project> {
        let mut f = File::open(path.join("info.json"))?;
        let mut buffer = String::new();

        f.read_to_string(&mut buffer)?;
        let mut proj: Project = serde_json::from_str(&buffer)?;
        proj.location = path.to_path_buf().clone();

        Ok(proj)
    }

    fn is_folder_a_project(path: &Path) -> bool {
        path.join("info.json").exists()
    }

    // Opens directory picker
    pub fn choose_project() -> Result<Project> {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            let mut valid = Project::is_folder_a_project(path.as_path());
            if !valid {
                if fs::read_dir(path.as_path()).unwrap().count() > 0 {
                    if rfd::MessageDialog::new()
                        .set_title("Create project in folder with files")
                        .set_description("Are you sure you want to create a project in a folder which already contain files")
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                        == rfd::MessageDialogResult::Yes
                        {
                            Project::create_project(&path)?;
                            valid = true;
                        }
                } else {
                    Project::create_project(&path)?;
                    valid = true;
                }
            }
            if !valid {
                return Err(anyhow::format_err!(""));
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
        info!("Saved project");
        Ok(())
    }

    pub fn add_awc_pack(&mut self, pack: AwcPack) {
        self.awc_info.push(pack);
        self.awc_info.sort();
    }

    pub fn generate_awc_file(&self, awc_pack_index: usize) -> Result<()> {
        let awc_pack = &self.awc_info[awc_pack_index];
        fs::create_dir_all(
            self.location
                .join(format!("output/awc/.packs/{}/", &awc_pack.name)),
        )?;

        let awc_xml: awc::AwcXML = match awc_pack.pack_type {
            awc::AwcPackType::Simple => {
                let mut streams = vec![];

                for ele in &awc_pack.entries {
                    ele.generate_splitted_variant(&self.location);
                    let entry_streams = ele.to_xml_stream();
                    streams.extend(entry_streams);
                }

                AwcXML {
                    version: xml::Value::new(1),
                    chunk_indices: xml::Value::new("True".to_string()),
                    streams: xml::ItemList { item: streams },
                }
            }
            awc::AwcPackType::Radio => {
                todo!();
            }
        };

        let serialized = xml::serialize_str(&awc_xml)?;

        let mut f = File::create(
            self.location
                .join(format!("output/awc/.packs/{}.xml", &awc_pack.name)),
        )?;
        f.write_all(serialized.as_bytes())?;

        Ok(())
    }
}

pub fn add_to_recent_projects(frame: &mut eframe::Frame, path: PathBuf) {
    let storage = frame.storage_mut().unwrap();
    let recent_projects_str = storage.get_string("project_history");
    let mut recent_projects = Vec::<PathBuf>::new();
    if let Some(paths) = recent_projects_str {
        recent_projects = serde_json::from_str(&paths).unwrap();
    }
    recent_projects.insert(0, path);
    recent_projects.truncate(10);
    storage.set_string(
        "project_history",
        serde_json::to_string(&recent_projects).unwrap(),
    );
}
