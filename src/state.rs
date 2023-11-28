use crate::components::{awc_generator, occl_generator};
use strum::EnumIter;

pub struct LoadedTabs {
    pub awc_generator: awc_generator::AwcGenerator,
    pub occl_generator: occl_generator::OcclGenerator,
}

impl Default for LoadedTabs {
    fn default() -> Self {
        Self {
            awc_generator: awc_generator::AwcGenerator {},
            occl_generator: occl_generator::OcclGenerator {},
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter)]
pub enum MenuEntry {
    AwcGenerator,
    OcclGenerator,
}

impl MenuEntry {
    pub fn label(&self) -> &str {
        match self {
            MenuEntry::AwcGenerator => "AWC Generator",
            MenuEntry::OcclGenerator => "Audio Occlusion",
        }
    }
    pub fn render_entry<'a>(&'a self, tab_store: &'a mut LoadedTabs) -> &mut dyn eframe::App {
        match self {
            MenuEntry::AwcGenerator => &mut tab_store.awc_generator as &mut dyn eframe::App,
            MenuEntry::OcclGenerator => &mut tab_store.occl_generator as &mut dyn eframe::App,
        }
    }
}

pub struct State {
    pub active_menu: MenuEntry,
}

impl Default for State {
    fn default() -> Self {
        Self {
            active_menu: MenuEntry::AwcGenerator,
        }
    }
}

impl State {
    pub fn change_menu(&mut self, entry: MenuEntry) {
        self.active_menu = entry;
    }
}
