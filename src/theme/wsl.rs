use crate::os;
use crate::state::State;
use alloc::string::String;
use alloc::vec::Vec;

const PROFILE_KEY: &str = "wsl_profile";
const USERS: &str = "/mnt/c/Users";
const SHARED_FOLDERS: [&str; 4] = ["Public", "Default", "Default User", "All Users"];

pub fn profile(state: &mut State) -> Option<String> {
    if let Some(known) = state.get(PROFILE_KEY).filter(|path| os::exists(path)) {
        return Some(String::from(known));
    }
    let path = os::env("PATH").unwrap_or_default();
    let found = profile_in_path(path.split(':')).or_else(|| profile_with_terminal(USERS))?;
    state.set(PROFILE_KEY, &found);
    Some(found)
}

pub fn profile_in_path<'a>(entries: impl IntoIterator<Item = &'a str>) -> Option<String> {
    entries.into_iter().find_map(|entry| {
        let parts: Vec<&str> = entry.split('/').collect();
        let at = parts.windows(3).position(|window| {
            is_drive(window[0]) && window[1].eq_ignore_ascii_case("Users") && is_person(window[2])
        })?;
        let profile = parts[..at + 3].join("/");
        os::exists(&profile).then_some(profile)
    })
}

pub fn profile_with_terminal(users: &str) -> Option<String> {
    os::list(users, "")
        .into_iter()
        .filter(|name| is_person(name))
        .map(|name| os::join(users, &[&name]))
        .find(|profile| {
            let packages = os::join(profile, &["AppData", "Local", "Packages"]);
            !os::list(&packages, "Microsoft.WindowsTerminal").is_empty()
        })
}

fn is_drive(part: &str) -> bool {
    part.len() == 1 && part.bytes().all(|b| b.is_ascii_alphabetic())
}

fn is_person(name: &str) -> bool {
    !name.is_empty()
        && !SHARED_FOLDERS
            .iter()
            .any(|shared| shared.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::Sandbox;

    const TERMINAL: &str = "Microsoft.WindowsTerminal_8wekyb3d8bbwe";

    #[test]
    fn the_windows_profile_comes_from_path_or_from_the_terminal_package() {
        let sandbox = Sandbox::new("wsl-profile");
        let users = sandbox.path("mnt/c/Users");
        let package =
            |person: &str| os::join(&users, &[person, "AppData", "Local", "Packages", TERMINAL]);
        std::fs::create_dir_all(package("Public")).unwrap();
        assert_eq!(profile_with_terminal(&users), None);
        std::fs::create_dir_all(package("alice")).unwrap();
        assert_eq!(
            profile_with_terminal(&users),
            Some(os::join(&users, &["alice"]))
        );
        let root = sandbox.dir().replace('\\', "/");
        let apps = format!("{root}/mnt/c/Users/alice/AppData/Local/Microsoft/WindowsApps");
        let alice = Some(format!("{root}/mnt/c/Users/alice"));
        assert_eq!(profile_in_path(["/usr/bin", apps.as_str()]), alice);
        assert_eq!(
            profile_in_path(["/mnt/c/Users/Public/bin", "/mnt/d/Users"]),
            None
        );
    }
}
