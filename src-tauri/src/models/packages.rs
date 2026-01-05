#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub location: String,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectDependency {
    pub name: String,
    pub version_spec: String,
    pub status: DependencyStatus,
    pub installed_version: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyStatus {
    Installed,
    Missing,
    VersionMismatch,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipCommand {
    pub action: String,
    pub package: String,
    pub version: Option<String>,
}