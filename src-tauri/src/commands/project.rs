use crate::models::ProjectDependency;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[tauri::command]
pub async fn detect_project_files(project_path: String) -> Result<Vec<String>, String> {
    let root = PathBuf::from(&project_path);

    let mut files = Vec::new();

    if root.join("pyproject.toml").exists() {
        files.push("pyproject.toml".to_string());
    }
    if root.join("requirements.txt").exists() {
        files.push("requirements.txt".to_string());
    }
    if root.join("requirements-dev.txt").exists() {
        files.push("requirements-dev.txt".to_string());
    }
    if root.join("setup.py").exists() {
        files.push("setup.py".to_string());
    }
    if root.join("Pipfile").exists() {
        files.push("Pipfile".to_string());
    }
    if root.join("poetry.lock").exists() {
        files.push("poetry.lock".to_string());
    }

    Ok(files)
}

#[tauri::command]
pub async fn parse_requirements(
    project_path: String,
    python_path: String,
) -> Result<Vec<ProjectDependency>, String> {
    let root = PathBuf::from(&project_path);
    let mut dependencies = Vec::new();

    // Check for pyproject.toml first
    if let Ok(deps) = parse_pyproject_toml(&root, &python_path) {
        dependencies.extend(deps);
    }

    // Check for requirements.txt
    if let Ok(deps) = parse_requirements_txt(&root.join("requirements.txt"), &python_path) {
        dependencies.extend(deps);
    }

    // Check for requirements-dev.txt
    if let Ok(deps) = parse_requirements_txt(&root.join("requirements-dev.txt"), &python_path) {
        dependencies.extend(deps);
    }

    // Get installed packages to check status
    let installed = get_installed_packages(&python_path)?;

    for dep in &mut dependencies {
        if let Some(inst_version) = installed.get(&dep.name.to_lowercase()) {
            dep.installed_version = Some(inst_version.clone());

            if dep.version_spec.is_empty() || dep.version_spec == "*" {
                dep.status = crate::models::DependencyStatus::Installed;
            } else if is_version_match(inst_version, &dep.version_spec) {
                dep.status = crate::models::DependencyStatus::Installed;
            } else {
                dep.status = crate::models::DependencyStatus::VersionMismatch;
            }
        } else {
            dep.status = crate::models::DependencyStatus::Missing;
        }
    }

    Ok(dependencies)
}

fn parse_pyproject_toml(
    root: &PathBuf,
    _python_path: &str,
) -> Result<Vec<ProjectDependency>, String> {
    let pyproject_path = root.join("pyproject.toml");

    let contents = fs::read_to_string(pyproject_path)
        .map_err(|e| format!("Cannot read pyproject.toml: {}", e))?;

    let mut dependencies = Vec::new();

    // Simple TOML parsing for dependencies
    let mut in_dependencies = false;

    for line in contents.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("[project]") {
            in_dependencies = true;
            continue;
        }

        if trimmed.starts_with("[") && trimmed != "[project]" {
            in_dependencies = false;
        }

        if in_dependencies && trimmed.starts_with("dependencies") {
            if let Some(content) = trimmed.strip_prefix("dependencies").and_then(|s| s.trim().strip_prefix("=")) {
                let content = content.trim().trim_matches('[').trim_matches(']').trim_matches('"');

                for dep in content.split(',') {
                    if let Some((name, spec)) = parse_requirement_string(dep.trim()) {
                        dependencies.push(ProjectDependency {
                            name,
                            version_spec: spec,
                            status: crate::models::DependencyStatus::Missing,
                            installed_version: None,
                        });
                    }
                }
            }
        }
    }

    Ok(dependencies)
}

fn parse_requirements_txt(
    path: &PathBuf,
    _python_path: &str,
) -> Result<Vec<ProjectDependency>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read requirements file: {}", e))?;

    let mut dependencies = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip flags
        if trimmed.starts_with('-') {
            continue;
        }

        if let Some((name, spec)) = parse_requirement_string(trimmed) {
            dependencies.push(ProjectDependency {
                name,
                version_spec: spec,
                status: crate::models::DependencyStatus::Missing,
                installed_version: None,
            });
        }
    }

    Ok(dependencies)
}

fn parse_requirement_string(req: &str) -> Option<(String, String)> {
    // Handle various formats: package, package==1.0, package>=1.0, package[extra]==1.0

    if let Some(idx) = req.find("==") {
        let name = req[..idx].trim().split('[').next()?.to_string();
        let version = req[idx + 2..].trim().to_string();
        return Some((name, version));
    }

    if let Some(idx) = req.find(">=") {
        let name = req[..idx].trim().split('[').next()?.to_string();
        let version = format!(">={}", req[idx + 2..].trim());
        return Some((name, version));
    }

    if let Some(idx) = req.find("<=") {
        let name = req[..idx].trim().split('[').next()?.to_string();
        let version = format!("<={}", req[idx + 2..].trim());
        return Some((name, version));
    }

    if let Some(idx) = req.find(">") {
        let name = req[..idx].trim().split('[').next()?.to_string();
        let version = format!(">{}", req[idx + 1..].trim());
        return Some((name, version));
    }

    if let Some(idx) = req.find("<") {
        let name = req[..idx].trim().split('[').next()?.to_string();
        let version = format!("<{}", req[idx + 1..].trim());
        return Some((name, version));
    }

    // Just package name
    let name = req.split('[').next()?.trim().to_string();
    Some((name, "*".to_string()))
}

fn get_installed_packages(python_path: &str) -> Result<std::collections::HashMap<String, String>, String> {
    let output = Command::new(python_path)
        .args(&["-m", "pip", "list", "--format=json"])
        .output()
        .map_err(|e| format!("Failed to get installed packages: {}", e))?;

    if !output.status.success() {
        return Ok(std::collections::HashMap::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<crate::models::InstalledPackage> = serde_json::from_str(&stdout)
        .unwrap_or_default();

    let mut map = std::collections::HashMap::new();
    for pkg in packages {
        map.insert(pkg.name.to_lowercase(), pkg.version);
    }

    Ok(map)
}

fn is_version_match(installed: &str, spec: &str) -> bool {
    if spec == "*" || spec.is_empty() {
        return true;
    }

    if spec.starts_with("==") {
        return installed == spec.trim_start_matches("==");
    }

    if spec.starts_with(">=") {
        return compare_versions(installed, spec.trim_start_matches(">=")) >= std::cmp::Ordering::Equal;
    }

    if spec.starts_with("<=") {
        return compare_versions(installed, spec.trim_start_matches("<=")) <= std::cmp::Ordering::Equal;
    }

    if spec.starts_with(">") {
        return compare_versions(installed, spec.trim_start_matches(">")) == std::cmp::Ordering::Greater;
    }

    if spec.starts_with("<") {
        return compare_versions(installed, spec.trim_start_matches("<")) == std::cmp::Ordering::Less;
    }

    true
}

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();

    for (ap, bp) in a_parts.iter().zip(b_parts.iter()) {
        if ap != bp {
            return ap.cmp(bp);
        }
    }

    a_parts.len().cmp(&b_parts.len())
}
