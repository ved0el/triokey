//! The version check: asks GitHub where its "latest release" page redirects and compares that tag with the running
//! version (docs/ARCHITECTURE.md, 12.15 Version check). It never downloads or installs anything.

use triokey_shared::{RELEASES, REPOSITORY};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// A newer version waits: its number, digits and dots only.
    Newer(String),
    Latest,
    Failed,
}

pub const CURL: &str = "/usr/bin/curl";
/// Seconds curl may take in all.
pub const MAX_TIME: &str = "20";

/// curl's arguments, passed without a shell. `-q` first: the user's `.curlrc` is not read. No `-L`: only the redirect
/// address is printed, nothing is followed or saved. No body, no query, nothing about the machine or the version.
pub fn curl_args() -> [&'static str; 12] {
    [
        "-q",
        "--silent",
        "--proto",
        "=https",
        "--max-time",
        MAX_TIME,
        "--user-agent",
        "TrioKey-updater",
        "--output",
        "/dev/null",
        "--write-out",
        "%{redirect_url}",
    ]
}

/// The address asked, after `curl_args`.
pub const ADDRESS: &str = RELEASES;

fn parts(v: &str) -> Option<Vec<u64>> {
    v.split('.').map(|p| if !p.is_empty() && p.len() <= 9 && p.bytes().all(|b| b.is_ascii_digit()) { p.parse().ok() } else { None }).collect()
}

/// `redirect`: where `ADDRESS` redirects to (curl's output); `current`: the running version.
pub fn answer(redirect: &str, current: &str) -> Answer {
    let tag = redirect.trim().strip_prefix(REPOSITORY).and_then(|r| r.strip_prefix("/releases/tag/v"));
    let (Some(tag), Some(cur)) = (tag, parts(current)) else { return Answer::Failed };
    let Some(new) = parts(tag) else { return Answer::Failed };
    let at = |v: &[u64], i: usize| v.get(i).copied().unwrap_or(0);
    let newer = (0..new.len().max(cur.len())).map(|i| at(&new, i).cmp(&at(&cur, i))).find(|o| o.is_ne()) == Some(std::cmp::Ordering::Greater);
    if newer {
        Answer::Newer(tag.to_string())
    } else {
        Answer::Latest
    }
}

/// First automatic check this long after start, then once a day; after a failure, at most once an hour.
pub const START_DELAY: u64 = 60;
pub const DAY: u64 = 24 * 3600;
pub const RETRY: u64 = 3600;

/// Whether the 2-second timer starts an automatic check. `uptime`, `since_try`: seconds since start and since the last
/// try this run; `now`, `last`: Unix seconds now and of the last answer (`last_update_check`).
pub fn due(auto: bool, uptime: u64, since_try: Option<u64>, now: u64, last: u64) -> bool {
    auto && uptime >= START_DELAY && since_try.is_none_or(|t| t >= RETRY) && now.saturating_sub(last) >= DAY
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAG: &str = "https://github.com/ved0el/triokey/releases/tag/v";

    #[test]
    fn newer_latest_or_failed() {
        assert_eq!(answer(&format!("{TAG}0.3.1"), "0.3.0"), Answer::Newer("0.3.1".into()));
        assert_eq!(answer(&format!("{TAG}0.10.0\n"), "0.9.9"), Answer::Newer("0.10.0".into()));
        assert_eq!(answer(&format!("{TAG}1.0"), "0.3.0"), Answer::Newer("1.0".into()));
        assert_eq!(answer(&format!("{TAG}0.3.0"), "0.3.0"), Answer::Latest);
        assert_eq!(answer(&format!("{TAG}0.3"), "0.3.0"), Answer::Latest);
        assert_eq!(answer(&format!("{TAG}0.2.1"), "0.3.0"), Answer::Latest);
        for bad in [
            "",
            "https://github.com/ved0el/triokey/releases",
            "https://github.com/ved0el/triokey/releases/tag/0.4.0",
            "https://github.com/other/triokey/releases/tag/v0.4.0",
            "http://github.com/ved0el/triokey/releases/tag/v0.4.0",
            "https://github.com/ved0el/triokey/releases/tag/v0.4.0-rc1",
            "https://github.com/ved0el/triokey/releases/tag/v0.4..0",
            "https://github.com/ved0el/triokey/releases/tag/v0.4.0?x=1",
            "https://github.com/ved0el/triokey/releases/tag/v+1.0",
            "https://github.com/ved0el/triokey/releases/tag/v99999999999.0",
            "https://github.com/ved0el/triokey/releases/tag/v",
        ] {
            assert_eq!(answer(bad, "0.3.0"), Answer::Failed, "{bad}");
        }
        assert_eq!(answer(&format!("{TAG}0.4.0"), "dev"), Answer::Failed);
    }

    #[test]
    fn curl_asks_only_the_fixed_address() {
        let a = curl_args();
        assert_eq!(a[0], "-q");
        assert!(!a.contains(&"-L") && !a.contains(&"--location") && a.contains(&"--max-time"));
        assert_eq!(ADDRESS, "https://github.com/ved0el/triokey/releases/latest");
    }

    #[test]
    fn schedule() {
        let day = 1_800_000_000;
        assert!(due(true, 61, None, day, 0));
        assert!(!due(false, 61, None, day, 0));
        assert!(!due(true, 30, None, day, 0), "a minute after start");
        assert!(!due(true, 61, None, day, day - 100), "answered today");
        assert!(due(true, 61, None, day, day - DAY));
        assert!(!due(true, 4000, Some(100), day, 0), "tried a moment ago");
        assert!(due(true, 4000, Some(RETRY), day, 0));
        assert!(!due(true, 61, None, day, day + 50), "clock set back");
    }
}
