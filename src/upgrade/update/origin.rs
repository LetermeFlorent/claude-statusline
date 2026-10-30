use alloc::format;
use alloc::string::String;

const REPO: &str = "LetermeFlorent/claude-statusline";
pub const HTTPS_ONLY: &str = "=https";
const GITHUB_API: &str = "https://api.github.com";
const GITHUB: &str = "https://github.com";
#[cfg(debug_assertions)]
const SERVER_VARIABLE: &str = "STATUSLINE_UPDATE_SERVER";
#[cfg(debug_assertions)]
const LOCAL_SERVER: &str = "http://127.0.0.1:";
#[cfg(debug_assertions)]
const HTTP_ONLY: &str = "=http";

pub struct Origin {
    api: String,
    downloads: String,
    pub protocols: &'static str,
}

pub fn github() -> Origin {
    Origin {
        api: String::from(GITHUB_API),
        downloads: String::from(GITHUB),
        protocols: HTTPS_ONLY,
    }
}

#[cfg(not(debug_assertions))]
pub fn origin() -> Origin {
    github()
}

#[cfg(debug_assertions)]
pub fn origin() -> Origin {
    match crate::os::env(SERVER_VARIABLE).filter(|base| base.starts_with(LOCAL_SERVER)) {
        Some(base) => Origin {
            api: base.clone(),
            downloads: base,
            protocols: HTTP_ONLY,
        },
        None => github(),
    }
}

impl Origin {
    pub fn latest(&self) -> String {
        format!("{}/repos/{}/releases/latest", self.api, REPO)
    }

    pub fn download(&self, tag: &str, name: &str) -> String {
        let downloads = &self.downloads;
        format!("{}/{}/releases/download/{}/{}", downloads, REPO, tag, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn releases_come_from_github_over_https() {
        let origin = github();
        let api = "https://api.github.com/repos/LetermeFlorent/claude-statusline/releases/latest";
        let download =
            "https://github.com/LetermeFlorent/claude-statusline/releases/download/v9.0.0/a";
        assert_eq!(origin.latest(), api);
        assert_eq!(origin.download("v9.0.0", "a"), download);
        assert_eq!(origin.protocols, HTTPS_ONLY);
    }
}
