use crate::models::{PyPIPackage, PackageMetadata};
use std::collections::HashMap;

const PYPI_API_BASE: &str = "https://pypi.org/pypi";

#[tauri::command]
pub async fn search_pypi(query: String) -> Result<Vec<PyPIPackage>, String> {
    // Using json API endpoint with XML-RPC style search
    let url = format!("{}/{}/json", PYPI_API_BASE, query);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch from PyPI: {}", e))?;

    if response.status() == 404 {
        return Ok(vec![]);
    }

    if !response.status().is_success() {
        return Err(format!("PyPI error: {}", response.status()));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let info = &data["info"];

    let package = PyPIPackage {
        name: info["name"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        version: info["version"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        summary: info["summary"].as_str().map(|s| s.to_string()),
        home_page: info["home_page"].as_str().map(|s| s.to_string()),
        author: info["author"].as_str().map(|s| s.to_string()),
        author_email: info["author_email"].as_str().map(|s| s.to_string()),
        license: info["license"].as_str().map(|s| s.to_string()),
    };

    // For search, we return single result. For multiple, would use different endpoint
    Ok(vec![package])
}

#[tauri::command]
pub async fn get_package_info(package_name: String) -> Result<(PackageMetadata, Vec<String>), String> {
    let url = format!("{}/{}/json", PYPI_API_BASE, package_name);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch from PyPI: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Package not found or PyPI error: {}", response.status()));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let info = &data["info"];

    let metadata = PackageMetadata {
        name: info["name"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        version: info["version"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        summary: info["summary"].as_str().map(|s| s.to_string()),
        description: info["description"].as_str().map(|s| s.to_string()),
        home_page: info["home_page"].as_str().map(|s| s.to_string()),
        author: info["author"].as_str().map(|s| s.to_string()),
        author_email: info["author_email"].as_str().map(|s| s.to_string()),
        license: info["license"].as_str().map(|s| s.to_string()),
        requires_python: info["requires_python"].as_str().map(|s| s.to_string()),
        classifiers: info["classifiers"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
    };

    let releases: Vec<String> = data["releases"]
        .as_object()
        .map(|obj| {
            let mut versions: Vec<_> = obj.keys().map(|k| k.clone()).collect();
            versions.sort_by(|a, b| {
                // Sort versions in descending order (newest first)
                compare_versions(b, a)
            });
            versions
        })
        .unwrap_or_default();

    Ok((metadata, releases))
}

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<&str> = a.split('.').collect();
    let b_parts: Vec<&str> = b.split('.').collect();

    for (ap, bp) in a_parts.iter().zip(b_parts.iter()) {
        let a_num: u32 = ap.parse().unwrap_or(0);
        let b_num: u32 = bp.parse().unwrap_or(0);

        if a_num != b_num {
            return a_num.cmp(&b_num);
        }
    }

    a_parts.len().cmp(&b_parts.len())
}
p