use serde::{Deserialize, Serialize};

/// A package being tracked, stored in the local SQLite database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackedPackage {
    pub name: String,
    pub latest_version: String,
    pub downloads: u64,
    pub daily: u64,
    pub last_updated: String,
}

/// Response from Packagist package details API.
#[derive(Debug, Deserialize)]
pub struct PackagistPackageDetails {
    #[serde(rename = "package")]
    pub package: PackageDetail,
}

#[derive(Debug, Deserialize)]
pub struct PackageDetail {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "time")]
    pub last_updated: Option<String>,
    #[serde(rename = "downloads")]
    pub downloads: Option<DownloadStats>,
    #[serde(rename = "versions")]
    pub versions: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct DownloadStats {
    #[serde(rename = "total")]
    pub total: Option<u64>,
    #[serde(rename = "daily")]
    pub daily: Option<u64>,
}

/// A daily snapshot of a package's statistics.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub date: String,
    pub name: String,
    pub latest_version: String,
    pub downloads: u64,
    pub daily: u64,
}

/// Extract the latest stable version from the versions JSON map.
/// Returns the highest version string (lexicographically sorted, excluding dev branches).
pub fn latest_version(versions: &serde_json::Value) -> String {
    versions
        .as_object()
        .and_then(|obj| obj.keys().filter(|k| !k.starts_with("dev-")).max())
        .cloned()
        .unwrap_or_else(|| "unknown".to_string())
}
