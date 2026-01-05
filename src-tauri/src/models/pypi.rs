#[derive(Debug, Serialize, Deserialize)]
pub struct PyPISearchResult {
    pub results: Vec<PyPIPackage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyPIPackage {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub home_page: Option<String>,
    pub author: Option<String>,
    pub author_email: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PyPIPackageInfo {
    pub info: PackageMetadata,
    pub releases: std::collections::HashMap<String, Vec<ReleaseInfo>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub home_page: Option<String>,
    pub author: Option<String>,
    pub author_email: Option<String>,
    pub license: Option<String>,
    pub requires_python: Option<String>,
    pub classifiers: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub upload_time: Option<String>,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_python: bool,
    pub is_venv: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipLogEntry {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}
