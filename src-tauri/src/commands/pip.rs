use crate::models::InstalledPackage;
use crate::state::AppState;
use std::process::Command;
use std::sync::Mutex;
use tauri::{State, Emitter};
use std::io::{BufRead, BufReader};

#[tauri::command]
pub async fn list_installed_packages(
    python_path: String,
) -> Result<Vec<InstalledPackage>, String> {
    let output = Command::new(&python_path)
        .args(&["-m", "pip", "list", "--format=json"])
        .output()
        .map_err(|e| format!("Failed to list packages: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<InstalledPackage> = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse pip output: {}", e))?;

    Ok(packages)
}

#[tauri::command]
pub async fn install_package(
    python_path: String,
    package: String,
    version: Option<String>,
    window: tauri::Window,
) -> Result<String, String> {
    let package_spec = if let Some(v) = version {
        format!("{}=={}", package, v)
    } else {
        package
    };

    execute_pip_command(&python_path, &["install", &package_spec], &window)
}

#[tauri::command]
pub async fn uninstall_package(
    python_path: String,
    package: String,
    window: tauri::Window,
) -> Result<String, String> {
    execute_pip_command(&python_path, &["uninstall", "-y", &package], &window)
}

#[tauri::command]
pub async fn upgrade_package(
    python_path: String,
    package: String,
    window: tauri::Window,
) -> Result<String, String> {
    execute_pip_command(
        &python_path,
        &["install", "--upgrade", &package],
        &window,
    )
}

#[tauri::command]
pub async fn downgrade_package(
    python_path: String,
    package: String,
    version: String,
    window: tauri::Window,
) -> Result<String, String> {
    execute_pip_command(
        &python_path,
        &["install", &format!("{}=={}", package, version)],
        &window,
    )
}

fn execute_pip_command(
    python_path: &str,
    args: &[&str],
    window: &tauri::Window,
) -> Result<String, String> {
    let mut cmd = Command::new(python_path);
    cmd.arg("-m").arg("pip");
    cmd.args(args);

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute pip: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture stdout")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("Failed to capture stderr")?;

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let mut output = String::new();

    // Spawn threads to read both streams
    let window_clone = window.clone();
    let stdout_handle = std::thread::spawn(move || {
        for line in stdout_reader.lines() {
            if let Ok(line) = line {
                let _ = window_clone.emit("pip-log", &line);
            }
        }
    });

    let window_clone = window.clone();
    let stderr_handle = std::thread::spawn(move || {
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                let _ = window_clone.emit("pip-log", format!("[ERROR] {}", line));
            }
        }
    });

    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for pip: {}", e))?;

    // Wait for log threads
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    if status.success() {
        Ok("Command completed successfully".to_string())
    } else {
        Err(format!("pip command failed with status: {}", status))
    }
}
