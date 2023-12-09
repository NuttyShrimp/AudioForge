use std::{cell::RefCell, rc::Rc};

use crate::{
    components::{awc_generator, occl_generator, project_selector},
    project_mgmt::project::Project,
};
use strum::EnumIter;

pub struct LoadedTabs {
    pub awc_generator: awc_generator::AwcGenerator,
    pub occl_generator: occl_generator::OcclGenerator,
    pub project_selector: project_selector::ProjectSelector,
}

impl LoadedTabs {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self {
            awc_generator: awc_generator::AwcGenerator::new(state.clone()),
            occl_generator: occl_generator::OcclGenerator {},
            project_selector: project_selector::ProjectSelector::new(state.clone()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter)]
pub enum MenuEntry {
    ProjectSelector,
    AwcGenerator,
    OcclGenerator,
}

impl MenuEntry {
    pub fn label(&self) -> Option<&str> {
        match self {
            MenuEntry::ProjectSelector => None,
            MenuEntry::AwcGenerator => Some("AWC Generator"),
            MenuEntry::OcclGenerator => Some("Audio Occlusion"),
        }
    }
    pub fn render_entry<'a>(&'a self, tab_store: &'a mut LoadedTabs) -> &mut dyn eframe::App {
        match self {
            MenuEntry::ProjectSelector => &mut tab_store.project_selector as &mut dyn eframe::App,
            MenuEntry::AwcGenerator => &mut tab_store.awc_generator as &mut dyn eframe::App,
            MenuEntry::OcclGenerator => &mut tab_store.occl_generator as &mut dyn eframe::App,
        }
    }
}

pub struct State {
    pub active_menu: MenuEntry,
    pub active_project: Option<Project>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            active_menu: MenuEntry::ProjectSelector,
            active_project: None,
        }
    }
}

impl State {
    pub fn change_menu(&mut self, entry: MenuEntry) {
        if self.active_project.is_none() {
            return;
        }
        self.active_menu = entry;
    }

    pub fn set_project(&mut self, project: Project) {
        self.active_project = Some(project);
    }

    pub fn close_project(&mut self) {
        self.active_project = None;
    }
}
