//! Updates from published releases, through `cargo-packager-updater`.
//!
//! The release workflow publishes a `latest.json` manifest with every release
//! (release candidates included) and signs every installer with the project's
//! updater key. Where releases are listed and which public key to trust are
//! compiled in from the environment of the build (`FLOETASK_UPDATE_RELEASES`,
//! the GitHub API URL of the repository's releases, and
//! `FLOETASK_UPDATE_PUBLIC_KEY`), so only release builds from the workflow
//! can update themselves. A build without them reports that updates are not
//! configured.
//!
//! Installing a version reads that release's own manifest, so the same path
//! updates to a newer release and rolls back to an older one.

use std::time::Duration;

use cargo_packager_updater::reqwest::blocking::Client;
use cargo_packager_updater::reqwest::header::{ACCEPT, USER_AGENT};
use cargo_packager_updater::semver::Version;
use cargo_packager_updater::url::Url;
use cargo_packager_updater::{Config, UpdaterBuilder, WindowsConfig, WindowsUpdateInstallMode};
use serde::Deserialize;

use floetask_application::AppError;
use floetask_application::ports::{PublishedRelease, Updater};

/// How long a check or download may take before it gives up.
const TIMEOUT: Duration = Duration::from_secs(60);

/// The updater manifest the release workflow attaches to every release.
const MANIFEST: &str = "latest.json";

/// How many releases one check lists: plenty for updates and rollbacks.
const PAGE_SIZE: u8 = 50;

pub struct ReleaseUpdater {
    current: Version,
    releases_url: Option<String>,
    pubkey: Option<String>,
}

/// A release as the GitHub API lists it; only the fields used here.
#[derive(Debug, Deserialize)]
struct ReleaseDto {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    body: Option<String>,
    #[serde(default)]
    assets: Vec<AssetDto>,
}

#[derive(Debug, Deserialize)]
struct AssetDto {
    name: String,
    browser_download_url: String,
}

/// A release that can be installed: its version and where its manifest is.
#[derive(Debug, Clone, PartialEq)]
struct Installable {
    version: Version,
    prerelease: bool,
    notes: Option<String>,
    manifest: String,
}

impl ReleaseUpdater {
    /// Uses the release list and public key this binary was built with.
    pub fn from_build_env() -> Self {
        Self::new(
            env!("CARGO_PKG_VERSION"),
            option_env!("FLOETASK_UPDATE_RELEASES"),
            option_env!("FLOETASK_UPDATE_PUBLIC_KEY"),
        )
    }

    fn new(current: &str, releases_url: Option<&str>, pubkey: Option<&str>) -> Self {
        let present = |value: Option<&str>| value.map(str::trim).filter(|v| !v.is_empty()).map(str::to_owned);
        Self {
            current: Version::parse(current).unwrap_or_else(|_| Version::new(0, 0, 0)),
            releases_url: present(releases_url),
            pubkey: present(pubkey),
        }
    }

    fn settings(&self) -> Result<(&str, &str), AppError> {
        match (&self.releases_url, &self.pubkey) {
            (Some(url), Some(pubkey)) => Ok((url, pubkey)),
            _ => Err(AppError::Other("updates are not configured for this build".to_owned())),
        }
    }

    /// Every installable release other than the running one, newest first.
    fn installable(&self) -> Result<Vec<Installable>, AppError> {
        let (url, _) = self.settings()?;
        let releases: Vec<ReleaseDto> = Client::new()
            .get(url)
            .query(&[("per_page", PAGE_SIZE)])
            // GitHub's API rejects requests without a user agent.
            .header(USER_AGENT, format!("floetask/{}", self.current))
            .header(ACCEPT, "application/vnd.github+json")
            .timeout(TIMEOUT)
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.json())
            .map_err(|e| AppError::Other(format!("could not list releases: {e}")))?;
        Ok(installable(releases, &self.current))
    }
}

/// The published releases with an updater manifest, without the running
/// version, newest first. Drafts and tags that are no version are left out.
fn installable(releases: Vec<ReleaseDto>, current: &Version) -> Vec<Installable> {
    let mut found: Vec<Installable> = releases
        .into_iter()
        .filter(|release| !release.draft)
        .filter_map(|release| {
            let version = Version::parse(release.tag_name.trim_start_matches('v')).ok()?;
            let manifest = release.assets.into_iter().find(|asset| asset.name == MANIFEST)?;
            Some(Installable {
                // A version with a pre-release part is a candidate, whatever
                // the release says.
                prerelease: release.prerelease || !version.pre.is_empty(),
                version,
                notes: release.body.filter(|body| !body.trim().is_empty()),
                manifest: manifest.browser_download_url,
            })
        })
        .filter(|release| release.version != *current)
        .collect();
    found.sort_by(|a, b| b.version.cmp(&a.version));
    found
}

impl Updater for ReleaseUpdater {
    fn is_configured(&self) -> bool {
        self.settings().is_ok()
    }

    fn releases(&self) -> Result<Vec<PublishedRelease>, AppError> {
        Ok(self
            .installable()?
            .into_iter()
            .map(|release| PublishedRelease {
                version: release.version.to_string(),
                prerelease: release.prerelease,
                notes: release.notes,
                newer: release.version > self.current,
            })
            .collect())
    }

    fn install(&self, version: &str) -> Result<(), AppError> {
        let (_, pubkey) = self.settings()?;
        let wanted = Version::parse(version).map_err(|e| AppError::Other(format!("invalid version {version}: {e}")))?;
        let release = self
            .installable()?
            .into_iter()
            .find(|release| release.version == wanted)
            .ok_or_else(|| AppError::Other(format!("floetask {version} is not published")))?;
        let manifest =
            Url::parse(&release.manifest).map_err(|e| AppError::Other(format!("invalid update manifest: {e}")))?;
        let config = Config {
            endpoints: vec![manifest],
            pubkey: pubkey.to_owned(),
            windows: Some(WindowsConfig {
                installer_args: None,
                // A progress bar only; the installer restarts the app.
                install_mode: Some(WindowsUpdateInstallMode::Passive),
            }),
        };
        // Exactly this version, so an older one installs too (a rollback).
        let update = UpdaterBuilder::new(self.current.clone(), config)
            .version_comparator(move |_, remote| remote.version == wanted)
            .timeout(TIMEOUT)
            .build()
            .and_then(|updater| updater.check())
            .map_err(|e| AppError::Other(format!("could not check floetask {version}: {e}")))?
            .ok_or_else(|| AppError::Other(format!("the manifest of floetask {version} lists another version")))?;
        update
            .download_and_install()
            .map_err(|e| AppError::Other(format!("could not install floetask {version}: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_without_release_list_or_key_are_not_configured() {
        assert!(!ReleaseUpdater::new("0.1.0", None, None).is_configured());
        assert!(
            !ReleaseUpdater::new("0.1.0", Some("https://api.github.com/repos/o/r/releases"), Some(" ")).is_configured()
        );
        let updater = ReleaseUpdater::new("0.1.0", Some("https://api.github.com/repos/o/r/releases"), Some("key"));
        assert!(updater.is_configured());
    }

    #[test]
    fn unconfigured_builds_explain_instead_of_calling_out() {
        let error = ReleaseUpdater::new("0.1.0", None, None).releases().unwrap_err();
        assert!(error.to_string().contains("not configured"));
    }

    fn dto(tag: &str, prerelease: bool, draft: bool, manifest: bool) -> ReleaseDto {
        ReleaseDto {
            tag_name: tag.to_owned(),
            draft,
            prerelease,
            body: Some(format!("notes of {tag}")),
            assets: manifest
                .then(|| AssetDto {
                    name: MANIFEST.to_owned(),
                    browser_download_url: format!("https://example.com/{tag}/latest.json"),
                })
                .into_iter()
                .collect(),
        }
    }

    #[test]
    fn lists_installable_releases_newest_first() {
        let current = Version::parse("0.4.0-rc.1").unwrap();
        let releases = vec![
            dto("v0.3.1", false, false, true),
            dto("v0.4.0-rc.2", true, false, true),
            dto("v0.4.0-rc.1", true, false, true),
            dto("v0.5.0", false, true, true),
            dto("v0.2.0", false, false, false),
            dto("nightly", true, false, true),
            dto("v0.4.0", false, false, true),
        ];
        let found = installable(releases, &current);
        let versions: Vec<String> = found.iter().map(|r| r.version.to_string()).collect();
        // Running, draft, manifest-less and non-version releases are left out.
        assert_eq!(versions, ["0.4.0", "0.4.0-rc.2", "0.3.1"]);
        assert!(found[1].prerelease && !found[0].prerelease);
        assert_eq!(found[2].manifest, "https://example.com/v0.3.1/latest.json");
    }

    #[test]
    fn reads_the_github_release_list() {
        let json = r#"[{"tag_name":"v0.4.0-rc.1","draft":false,"prerelease":true,"body":"",
            "assets":[{"name":"latest.json","browser_download_url":"https://x/latest.json","size":1}],
            "html_url":"https://x"}]"#;
        let releases: Vec<ReleaseDto> = serde_json::from_str(json).unwrap();
        let found = installable(releases, &Version::new(0, 3, 1));
        assert_eq!(found.len(), 1);
        assert!(found[0].prerelease && found[0].notes.is_none());
    }
}
