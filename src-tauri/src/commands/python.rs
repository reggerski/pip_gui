use crate::models::{PythonInstallation, VenvInfo};
use crate::state::AppState;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::State;
use std::sync::Mutex;

#[tauri::command]
pub async fn detect_python_installations() -> Result<Vec<PythonInstallation>, String> {
    let mut installations = Vec::new();

    #[cfg(target_os = "windows")]
    {
        // Check common Windows locations
        let paths = vec![
            "C:\\Python312\\python.exe",
            "C:\\Python311\\python.exe",
            "C:\\Python310\\python.exe",
            "C:\\Program Files\\Python312\\python.exe",
            "C:\\Program Files\\Python311\\python.exe",
            "C:\\Program Files (x86)\\Python312\\python.exe",
        ];

        for path_str in paths {
            if let Ok(inst) = validate_python_path_impl(PathBuf::from(path_str)) {
                if !installations.iter().any(|i| i.path == inst.path) {
                    installations.push(inst);
                }
            }
        }

        // Check User appdata
        if let Ok(appdata) = std::env::var("APPDATA") {
            let python_launcher = format!("{}\\Python\\Launcher\\python.exe", appdata);
            if Path::new(&python_launcher).exists() {
                if let Ok(inst) = validate_python_path_impl(PathBuf::from(python_launcher)) {
                    if !installations.iter().any(|i| i.path == inst.path) {
                        installations.push(inst);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let paths = vec![
            "/usr/local/bin/python3",
            "/usr/bin/python3",
            "/opt/homebrew/bin/python3",
            "/Library/Frameworks/Python.framework/Versions/Current/bin/python3",
        ];

        for path_str in paths {
            if let Ok(inst) = validate_python_path_impl(PathBuf::from(path_str)) {
                if !installations.iter().any(|i| i.path == inst.path) {
                    installations.push(inst);
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let paths = vec![
            "/usr/bin/python3",
            "/usr/local/bin/python3",
            "/usr/bin/python",
        ];

        for path_str in paths {
            if let Ok(inst) = validate_python_path_impl(PathBuf::from(path_str)) {
                if !installations.iter().any(|i| i.path == inst.path) {
                    installations.push(inst);
                }
            }
        }
    }

    if installations.is_empty() {
        Err("No Python installations found".to_string())
    } else {
        Ok(installations)
    }
}

#[tauri::command]
pub async fn validate_python_path(path: String) -> Result<PythonInstallation, String> {
    validate_python_path_impl(PathBuf::from(path))
}

#[tauri::command]
pub async fn select_python(
    path: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<PythonInstallation, String> {
    let installation = validate_python_path_impl(PathBuf::from(path))?;

    let mut app_state = state.lock().map_err(|e| format!("State lock error: {}", e))?;

    let selection = crate::state::PythonSelection {
        path: installation.path.clone(),
        version: installation.version.clone(),
        is_venv: installation.is_venv,
        venv_base: installation.venv_base.clone(),
    };

    app_state
        .save_python_selection(&selection)
        .map_err(|e| format!("Failed to save selection: {}", e))?;

    app_state.selected_python = Some(selection);

    Ok(installation)
}

#[tauri::command]
pub async fn get_selected_python(state: State<'_, Mutex<AppState>>) -> Result<Option<PythonInstallation>, String> {
    let app_state = state.lock().map_err(|e| format!("State lock error: {}", e))?;

    if let Some(selection) = &app_state.selected_python {
        Ok(Some(PythonInstallation {
            path: selection.path.clone(),
            version: selection.version.clone(),
            is_venv: selection.is_venv,
            venv_base: selection.venv_base.clone(),
            pip_version: get_pip_version_impl(&selection.path)?,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn get_pip_version(path: String) -> Result<String, String> {
    get_pip_version_impl(&PathBuf::from(path))
}

pub fn validate_python_path_impl(path: PathBuf) -> Result<PythonInstallation, String> {
    if !path.exists() {
        return Err(format!("Python executable not found: {:?}", path));
    }

    // Check if it's an executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&path).map_err(|e| format!("Cannot read path: {}", e))?;
        let mode = metadata.permissions().mode();
        if (mode & 0o111) == 0 {
            return Err("Path is not executable".to_string());
        }
    }

    // Get version
    let version_output = Command::new(&path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to run python: {}", e))?;

    let version_str = String::from_utf8_lossy(&version_output.stdout);
    let version = version_str
        .trim()
        .split(' ')
        .last()
        .unwrap_or("unknown")
        .to_string();

    // Check if it's a venv
    let is_venv = is_venv_python(&path);
    let venv_base = if is_venv {
        find_venv_base(&path)
    } else {
        None
    };

    // Verify pip is available
    let pip_test = Command::new(&path)
        .args(&["-m", "pip", "--version"])
        .output();

    if pip_test.is_err() {
        return Err("pip is not available in this Python installation".to_string());
    }

    Ok(PythonInstallation {
        path,
        version,
        is_venv,
        venv_base,
        pip_version: "".to_string(), // Will be fetched separately
    })
}

fn get_pip_version_impl(python_path: &PathBuf) -> Result<String, String> {
    let output = Command::new(python_path)
        .args(&["-m", "pip", "--version"])
        .output()
        .map_err(|e| format!("Failed to get pip version: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}

fn is_venv_python(python_path: &PathBuf) -> bool {
    // Check for pyvenv.cfg
    let parent = python_path.parent();
    if let Some(parent) = parent {
        let pyvenv_cfg = parent.join("pyvenv.cfg");
        return pyvenv_cfg.exists();
    }
    false
}

fn find_venv_base(python_path: &PathBuf) -> Option<PathBuf> {
    let mut current = python_path.parent()?;

    for _ in 0..5 {
        let pyvenv_cfg = current.join("pyvenv.cfg");
        if pyvenv_cfg.exists() {
            if let Ok(contents) = fs::read_to_string(&pyvenv_cfg) {
                for line in contents.lines() {
                    if let Some(home) = line.strip_prefix("home = ") {
                        return Some(PathBuf::from(home));
                    }
                }
            }
            return Some(current.to_path_buf());
        }

        current = current.parent()?;
    }

    None
}
