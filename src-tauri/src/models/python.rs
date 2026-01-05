use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PythonInstallation {
    pub path: PathBuf,
    pub version: String,
    pub is_venv: bool,
    pub venv_base: Option<PathBuf>,
    pub pip_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VenvInfo {
    pub path: PathBuf,
    pub python_path: PathBuf,
    pub home: String,
    pub prompt: Option<String>,
}
