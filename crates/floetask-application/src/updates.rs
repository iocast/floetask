//! Which published release to offer, and which ones a rollback can return
//! to, depending on the update channel. The updater only lists releases;
//! these rules decide.

use crate::ports::PublishedRelease;
use crate::settings::UpdateChannel;

/// How many earlier releases a rollback offers.
const ROLLBACK_LIMIT: usize = 10;

fn on_channel(release: &PublishedRelease, channel: UpdateChannel) -> bool {
    channel == UpdateChannel::Preview || !release.prerelease
}

/// The newest release on the channel that is newer than the running build.
/// `releases` is newest first, as the updater lists them.
pub fn offered(releases: &[PublishedRelease], channel: UpdateChannel) -> Option<&PublishedRelease> {
    releases
        .iter()
        .find(|release| release.newer && on_channel(release, channel))
}

/// The releases on the channel older than the running build, newest first.
/// On the stable channel this is how a preview build goes back to the last
/// release.
pub fn rollback_targets(releases: &[PublishedRelease], channel: UpdateChannel) -> Vec<&PublishedRelease> {
    releases
        .iter()
        .filter(|release| !release.newer && on_channel(release, channel))
        .take(ROLLBACK_LIMIT)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(version: &str, newer: bool) -> PublishedRelease {
        PublishedRelease {
            version: version.to_owned(),
            prerelease: version.contains("-rc."),
            notes: None,
            newer,
        }
    }

    /// Running 0.4.0-rc.1, newest first.
    fn published() -> Vec<PublishedRelease> {
        vec![
            release("0.4.0-rc.2", true),
            release("0.3.1", false),
            release("0.3.1-rc.1", false),
            release("0.3.0", false),
        ]
    }

    #[test]
    fn only_preview_offers_release_candidates() {
        let releases = published();
        assert_eq!(
            offered(&releases, UpdateChannel::Preview).map(|r| r.version.as_str()),
            Some("0.4.0-rc.2")
        );
        assert_eq!(offered(&releases, UpdateChannel::Stable), None);

        let mut with_release = releases.clone();
        with_release.insert(0, release("0.4.0", true));
        assert_eq!(
            offered(&with_release, UpdateChannel::Stable).map(|r| r.version.as_str()),
            Some("0.4.0")
        );
        assert_eq!(
            offered(&with_release, UpdateChannel::Preview).map(|r| r.version.as_str()),
            Some("0.4.0")
        );
    }

    #[test]
    fn rollback_goes_back_to_older_releases_on_the_channel() {
        let releases = published();
        let versions = |channel| -> Vec<String> {
            rollback_targets(&releases, channel)
                .iter()
                .map(|r| r.version.clone())
                .collect()
        };
        assert_eq!(versions(UpdateChannel::Stable), ["0.3.1", "0.3.0"]);
        assert_eq!(versions(UpdateChannel::Preview), ["0.3.1", "0.3.1-rc.1", "0.3.0"]);
    }

    #[test]
    fn rollback_offers_only_the_last_few_releases() {
        let releases: Vec<_> = (0..20)
            .rev()
            .map(|minor| release(&format!("0.{minor}.0"), false))
            .collect();
        let targets = rollback_targets(&releases, UpdateChannel::Stable);
        assert_eq!(targets.len(), ROLLBACK_LIMIT);
        assert_eq!(targets[0].version, "0.19.0");
    }
}
