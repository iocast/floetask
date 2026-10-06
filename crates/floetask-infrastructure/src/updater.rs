//! Updates from published releases, through `cargo-packager-updater`.
//!
//! The release workflow publishes a `latest.json` next to the installers and
//! signs every installer with the project's updater key. Which manifest to
//! ask and which public key to trust are compiled in from the environment of
//! the build (`FLOETASK_UPDATE_ENDPOINT`, `FLOETASK_UPDATE_PUBLIC_KEY`), so only
//! release builds from the workflow can update themselves. A build without
//! them reports that updates are not configured.

use std::time::Duration;

use cargo_packager_updater::semver::Version;
use cargo_packager_updater::url::Url;
use cargo_packager_updater::{Config, Update, UpdaterBuilder, WindowsConfig, WindowsUpdateInstallMode};

use floetask_application::AppError;
use floetask_application::ports::{AvailableUpdate, Updater};

/// How long a check or download may take before it gives up.
const TIMEOUT: Duration = Duration::from_secs(60);

pub struct ReleaseUpdater {
    current: Version,
    endpoint: Option<String>,
    pubkey: Option<String>,
}

impl ReleaseUpdater {
    /// Uses the endpoint and public key this binary was built with.
    pub fn from_build_env() -> Self {
        Self::new(
            env!("CARGO_PKG_VERSION"),
            option_env!("FLOETASK_UPDATE_ENDPOINT"),
            option_env!("FLOETASK_UPDATE_PUBLIC_KEY"),
        )
    }

    fn new(current: &str, endpoint: Option<&str>, pubkey: Option<&str>) -> Self {
        let present = |value: Option<&str>| value.map(str::trim).filter(|v| !v.is_empty()).map(str::to_owned);
        Self {
            current: Version::parse(current).unwrap_or_else(|_| Version::new(0, 0, 0)),
            endpoint: present(endpoint),
            pubkey: present(pubkey),
        }
    }

    fn find(&self) -> Result<Option<Update>, AppError> {
        let (Some(endpoint), Some(pubkey)) = (&self.endpoint, &self.pubkey) else {
            return Err(AppError::Other("updates are not configured for this build".to_owned()));
        };
        let endpoint = Url::parse(endpoint).map_err(|e| AppError::Other(format!("invalid update endpoint: {e}")))?;
        let config = Config {
            endpoints: vec![endpoint],
            pubkey: pubkey.clone(),
            windows: Some(WindowsConfig {
                installer_args: None,
                // A progress bar only; the installer restarts the app.
                install_mode: Some(WindowsUpdateInstallMode::Passive),
            }),
        };
        UpdaterBuilder::new(self.current.clone(), config)
            .timeout(TIMEOUT)
            .build()
            .and_then(|updater| updater.check())
            .map_err(|e| AppError::Other(format!("could not check for updates: {e}")))
    }
}

impl Updater for ReleaseUpdater {
    fn is_configured(&self) -> bool {
        self.endpoint.is_some() && self.pubkey.is_some()
    }

    fn check(&self) -> Result<Option<AvailableUpdate>, AppError> {
        Ok(self.find()?.map(|update| AvailableUpdate {
            version: update.version.trim_start_matches('v').to_owned(),
            notes: update.body.filter(|notes| !notes.trim().is_empty()),
        }))
    }

    fn install_latest(&self) -> Result<(), AppError> {
        let update = self
            .find()?
            .ok_or_else(|| AppError::Other("floetask is already up to date".to_owned()))?;
        update
            .download_and_install()
            .map_err(|e| AppError::Other(format!("could not install the update: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_without_endpoint_or_key_are_not_configured() {
        assert!(!ReleaseUpdater::new("0.1.0", None, None).is_configured());
        assert!(!ReleaseUpdater::new("0.1.0", Some("https://example.com/latest.json"), Some(" ")).is_configured());
        let updater = ReleaseUpdater::new("0.1.0", Some("https://example.com/latest.json"), Some("key"));
        assert!(updater.is_configured());
    }

    #[test]
    fn unconfigured_builds_explain_instead_of_calling_out() {
        let error = ReleaseUpdater::new("0.1.0", None, None).check().unwrap_err();
        assert!(error.to_string().contains("not configured"));
    }
}
