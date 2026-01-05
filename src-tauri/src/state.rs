use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use directories::ProjectDirs;
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PythonSelection {
    pub path: PathBuf,
    pub version: String,
    pub is_venv: bool,
    pub venv_base: Option<PathBuf>,
}

pub struct AppState {
    pub selected_python: Option<PythonSelection>,
    pub config_dir: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        let config_dir = ProjectDirs::from("com", "pip_gui", "pip_gui")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".pip_gui"));

        let mut app_state = AppState {
            selected_python: None,
            config_dir: config_dir.clone(),
        };

        // Try to load persisted Python selection
        if let Ok(selection) = app_state.load_python_selection() {
            app_state.selected_python = Some(selection);
        }

        app_state
    }

    pub fn load_python_selection(&self) -> Result<PythonSelection, Box<dyn std::error::Error>> {
        let config_file = self.config_dir.join("python_selection.json");
        let contents = fs::read_to_string(config_file)?;
        let selection: PythonSelection = serde_json::from_str(&contents)?;
        Ok(selection)
    }

    //pub fn save_python_selection(&self, selection: &PythonSelection) -> Result<(), Box<dyn std::error::Error>> {
    //   fs::create_dir_all(&self.config_dir)?;
    //    let config_file = self.config_dir.join("python_selection.json");
    //    let json = serde_json::to_string_pretty(selection)?;
    //    fs::write(config_file, json)?;
    //    Ok(())
    //}
}
