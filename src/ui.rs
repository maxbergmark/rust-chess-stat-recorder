use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use box_ui::BoxUI;

use crate::{
    util::{FileInfo, Progress},
    Error, Result,
};

mod box_ui;
mod file_progress;
mod util;

pub trait UserInterface {
    fn add_file(&mut self, file_info: &FileInfo);
    fn set_downloading(&mut self, filename: &str);
    fn set_processing(&mut self, filename: &str);
    fn hide_file(&mut self, filename: &str) -> Result<()>;
    fn set_error(&mut self, filename: &str, err: &Error);
    fn update_progress(&mut self, filename: &str, progress: Progress) -> Result<()>;
    fn complete_file(&mut self, filename: &str, progress: Progress) -> Result<()>;
    fn wait_for_exit(&mut self) -> Result<()>;
    fn exit(&mut self) -> Result<()>;
}

#[derive(Debug)]
pub enum UI {
    BoxUI(BoxUI),
    Empty,
}

impl UI {
    pub fn new(visible: bool) -> Result<Self> {
        if visible {
            BoxUI::new().map(Self::BoxUI)
        } else {
            Ok(Self::Empty)
        }
    }

    pub fn new_arc() -> Result<Arc<Mutex<Self>>> {
        Self::new(true).map(Mutex::new).map(Arc::new)
    }

    fn perform_ui_action(
        ui_mutex: &Arc<Mutex<Self>>,
        action: impl FnOnce(&mut Self) -> Result<()>,
    ) -> Result<()> {
        let mut ui = ui_mutex.lock()?;
        action(&mut ui)
    }

    pub fn update_progress(
        ui_mutex: &Arc<Mutex<Self>>,
        filename: &str,
        progress: Progress,
    ) -> Result<()> {
        Self::perform_ui_action(ui_mutex, |ui| ui.update_progress(filename, progress))
    }

    pub fn complete_file(
        ui_mutex: &Arc<Mutex<Self>>,
        filename: &str,
        progress: Progress,
    ) -> Result<()> {
        Self::perform_ui_action(ui_mutex, |ui| ui.complete_file(filename, progress))?;
        thread::sleep(Duration::from_secs(5));
        Self::perform_ui_action(ui_mutex, |ui| ui.hide_file(filename))
    }

    pub fn set_downloading(ui_mutex: &Arc<Mutex<Self>>, filename: &str) -> Result<()> {
        Self::perform_ui_action(ui_mutex, |ui| {
            ui.set_downloading(filename);
            Ok(())
        })
    }

    pub fn set_processing(ui_mutex: &Arc<Mutex<Self>>, filename: &str) -> Result<()> {
        Self::perform_ui_action(ui_mutex, |ui| {
            ui.set_processing(filename);
            Ok(())
        })
    }

    pub fn add_file(ui_mutex: &Arc<Mutex<Self>>, file_info: &FileInfo) -> Result<()> {
        Self::perform_ui_action(ui_mutex, |ui| {
            ui.add_file(file_info);
            Ok(())
        })
    }

    pub fn set_error(ui_mutex: &Arc<Mutex<Self>>, filename: &str, err: &Error) -> Result<()> {
        Self::perform_ui_action(ui_mutex, |ui| {
            ui.set_error(filename, err);
            Ok(())
        })
    }
}

impl UserInterface for UI {
    fn add_file(&mut self, file_info: &FileInfo) {
        if let Self::BoxUI(ui) = self {
            ui.add_file(file_info);
        }
    }

    fn set_downloading(&mut self, filename: &str) {
        if let Self::BoxUI(ui) = self {
            ui.set_downloading(filename);
        }
    }

    fn set_processing(&mut self, filename: &str) {
        if let Self::BoxUI(ui) = self {
            ui.set_processing(filename);
        }
    }

    fn set_error(&mut self, filename: &str, err: &Error) {
        if let Self::BoxUI(ui) = self {
            ui.set_error(filename, err);
        }
    }

    fn hide_file(&mut self, filename: &str) -> Result<()> {
        if let Self::BoxUI(ui) = self {
            ui.hide_file(filename)
        } else {
            Ok(())
        }
    }

    fn update_progress(&mut self, filename: &str, progress: Progress) -> Result<()> {
        if let Self::BoxUI(ui) = self {
            ui.update_progress(filename, progress)
        } else {
            Ok(())
        }
    }

    fn complete_file(&mut self, filename: &str, progress: Progress) -> Result<()> {
        if let Self::BoxUI(ui) = self {
            ui.complete_file(filename, progress)
        } else {
            Ok(())
        }
    }

    fn wait_for_exit(&mut self) -> Result<()> {
        match self {
            Self::BoxUI(ui) => ui.wait_for_exit(),
            Self::Empty => Ok(()),
        }
    }

    fn exit(&mut self) -> Result<()> {
        match self {
            Self::BoxUI(ui) => ui.exit(),
            Self::Empty => Ok(()),
        }
    }
}
