use crate::compact::files::project_folder;

const CWD: &str = "/work/sub";
const ROOT: &str = "/work";
const SHARE: &str = "//server/share";

fn local(dir: &str) -> bool {
    !dir.starts_with("//")
}

#[test]
fn working_folder_with_settings_comes_first() {
    let has = |_: &str| true;
    let got = project_folder([Some(CWD), Some(ROOT)], &local, &has);
    assert_eq!(got, Some(CWD));
}

#[test]
fn project_dir_is_used_when_the_working_folder_has_no_settings() {
    let has = |dir: &str| dir == ROOT;
    let got = project_folder([Some(CWD), Some(ROOT)], &local, &has);
    assert_eq!(got, Some(ROOT));
}

#[test]
fn working_folder_is_kept_when_no_folder_has_settings() {
    let has = |_: &str| false;
    let got = project_folder([Some(CWD), Some(ROOT)], &local, &has);
    assert_eq!(got, Some(CWD));
}

#[test]
fn project_dir_alone_is_used_without_a_working_folder() {
    let has = |_: &str| true;
    let got = project_folder([None, Some(ROOT)], &local, &has);
    assert_eq!(got, Some(ROOT));
}

#[test]
fn network_project_dir_is_never_probed() {
    let has = |dir: &str| {
        assert_ne!(dir, SHARE);
        false
    };
    let got = project_folder([Some(CWD), Some(SHARE)], &local, &has);
    assert_eq!(got, Some(CWD));
}

#[test]
fn network_working_folder_is_skipped_for_the_project_dir() {
    let has = |dir: &str| {
        assert_ne!(dir, SHARE);
        true
    };
    let got = project_folder([Some(SHARE), Some(ROOT)], &local, &has);
    assert_eq!(got, Some(ROOT));
}

#[test]
fn no_local_folder_gives_none() {
    let has = |_: &str| true;
    let got = project_folder([Some(SHARE), None], &local, &has);
    assert_eq!(got, None);
}
