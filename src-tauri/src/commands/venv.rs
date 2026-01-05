use crate::models::VenvInfo;
use std::fs;
use std::path::{Path, PathBuf};

#[tauri::command]
pub async fn detect_venvs(project_path: String) -> Result<Vec<VenvInfo>, String> {
    let root = PathBuf::from(project_path);

    if !root.is_dir() {
        return Err("Project path is not a directory".to_string());
    }

    let mut venvs = Vec::new();

    // Check standard venv names
    let venv_names = vec![".venv", "venv", "env", ".env"];

    for name in venv_names {
        let venv_path = root.join(name);
        if is_valid_venv(&venv_path) {
            if let Some(venv_info) = extract_venv_info(&venv_path) {
                venvs.push(venv_info);
            }
        }
    }

    // Scan subdirectories for pyvenv.cfg
    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let path = entry.path();
                    if is_valid_venv(&path) {
                        if let Some(venv_info) = extract_venv_info(&path) {
                            if !venvs.iter().any(|v| v.path == path) {
                                venvs.push(venv_info);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(venvs)
}

fn is_valid_venv(path: &Path) -> bool {
    let pyvenv_cfg = path.join("pyvenv.cfg");
    pyvenv_cfg.exists()
}

fn extract_venv_info(venv_path: &Path) -> Option<VenvInfo> {
    let pyvenv_cfg = venv_path.join("pyvenv.cfg");

    let mut home = String::new();
    let mut prompt = None;

    if let Ok(contents) = fs::read_to_string(&pyvenv_cfg) {
        for line in contents.lines() {
            if let Some(h) = line.strip_prefix("home = ") {
                home = h.to_string();
            } else if let Some(p) = line.strip_prefix("prompt = ") {
                prompt = Some(p.to_string());
            }
        }
    }

    let python_path = if cfg!(windows) {
        venv_path.join("Scripts").join("python.exe")
    } else {
        venv_path.join("bin").join("python")
    };

    if !python_path.exists() {
        return None;
    }

    Some(VenvInfo {
        path: venv_path.to_path_buf(),
        python_path,
        home,
        prompt,
    })
}
