use std::path::{Path, PathBuf};
use regex::Regex;

pub fn is_valid_package_name(name: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    re.is_match(name)
}

pub fn is_valid_version(version: &str) -> bool {
    let re = Regex::new(r"^\d+(\.\d+)*([a-zA-Z0-9\-\.]*)?$").unwrap();
    re.is_match(version)
}

pub fn normalize_package_name(name: &str) -> String {
    name.to_lowercase().replace('_', "-")
}

pub fn sanitize_path(path: &str) -> PathBuf {
    PathBuf::from(path)
}

pub fn is_safe_path(path: &Path) -> bool {
    // Prevent path traversal
    !path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
}
