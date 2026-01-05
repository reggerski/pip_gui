use crate::models::DirectoryEntry;
use std::fs;
use std::path::{Path, PathBuf};

#[tauri::command]
pub async fn list_directory(path: String) -> Result<Vec<DirectoryEntry>, String> {
    let dir_path = PathBuf::from(&path);

    if !dir_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let mut entries = Vec::new();

    match fs::read_dir(&dir_path) {
        Ok(read_dir) => {
            for entry in read_dir.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // Skip hidden files on Unix (except .venv)
                    #[cfg(unix)]
                    if name.starts_with('.') && name != ".venv" {
                        continue;
                    }

                    let is_dir = metadata.is_dir();
                    let is_python = is_python_executable(&path);
                    let is_venv = is_dir && is_venv_dir(&path);

                    entries.push(DirectoryEntry {
                        name,
                        path: path.to_string_lossy().to_string(),
                        is_dir,
                        is_python,
                        is_venv,
                    });
                }
            }
        }
        Err(e) => {
            return Err(format!("Cannot read directory: {}", e));
        }
    }

    // Sort: directories first, then by name
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(entries)
}

fn is_python_executable(path: &Path) -> bool {
    if path.is_file() {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        #[cfg(windows)]
        {
            return file_name.eq_ignore_ascii_case("python.exe")
                || file_name.eq_ignore_ascii_case("python");
        }

        #[cfg(not(windows))]
        {
            if file_name == "python" || file_name == "python3" {
                if let Ok(metadata) = fs::metadata(path) {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();
                    return (mode & 0o111) != 0;
                }
            }
        }
    }

    false
}

fn is_venv_dir(path: &Path) -> bool {
    path.join("pyvenv.cfg").exists()
}

#[tauri::command]
pub async fn get_home_directory() -> Result<String, String> {
    let home = dirs::home_dir()
        .ok_or("Cannot determine home directory")?
        .to_string_lossy()
        .to_string();
    Ok(home)
}

#[tauri::command]
pub async fn get_drives() -> Result<Vec<String>, String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::ffi::OsStr;

        let mut drives = Vec::new();
        let mask = unsafe { winapi::um::winbase::GetLogicalDrives() };

        for i in 0..26 {
            if (mask & (1 << i)) != 0 {
                let letter = (b'A' + i) as char;
                drives.push(format!("{}:\\", letter));
            }
        }

        Ok(drives)
    }

    #[cfg(not(windows))]
    {
        Ok(vec!["/".to_string()])
    }
}
