use anyhow::{Context, Result, bail};

use crate::model::{PackagistPackageDetails, TrackedPackage, latest_version};

const PACKAGIST_URL: &str = "https://packagist.org";

/// Fetch package details from Packagist and return a TrackedPackage.
pub async fn fetch_package(name: &str) -> Result<TrackedPackage> {
    let url = format!(
        "{}/packages/{}/{}.json",
        PACKAGIST_URL,
        name.split('/').next().unwrap_or(""),
        name
    );

    let resp = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to send request to Packagist for {}", name))?;

    if !resp.status().is_success() {
        bail!(
            "Packagist returned status {} for package {}",
            resp.status(),
            name
        );
    }

    let body = resp
        .text()
        .await
        .with_context(|| format!("Failed to read response body for {}", name))?;

    let details: PackagistPackageDetails = serde_json::from_str(&body)
        .with_context(|| format!("Failed to parse JSON for {}", name))?;

    let pkg = &details.package;
    let version = latest_version(&pkg.versions);
    let downloads = pkg.downloads.as_ref().and_then(|d| d.total).unwrap_or(0);
    let daily = pkg.downloads.as_ref().and_then(|d| d.daily).unwrap_or(0);
    let last_updated = pkg.last_updated.clone().unwrap_or_default();

    Ok(TrackedPackage {
        name: pkg.name.clone(),
        latest_version: version,
        downloads,
        daily,
        last_updated,
    })
}

#[cfg(test)]
mod tests {
    use crate::model::latest_version;

    #[test]
    fn latest_version_extracts_stable_version() {
        let versions = serde_json::json!({
            "v3.8.1": {},
            "v3.8.0": {},
            "dev-main": {}
        });
        assert_eq!(latest_version(&versions), "v3.8.1");
    }

    #[test]
    fn latest_version_returns_unknown_for_empty() {
        let versions = serde_json::json!({});
        assert_eq!(latest_version(&versions), "unknown");
    }

    #[test]
    fn latest_version_skips_dev_branches() {
        let versions = serde_json::json!({
            "dev-main": {},
            "dev-feature": {}
        });
        assert_eq!(latest_version(&versions), "unknown");
    }
}
