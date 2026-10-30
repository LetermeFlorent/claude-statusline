use super::on_network;
use super::wide::verbatim;
use crate::os::windows::process::system_dir;
use alloc::string::String;

#[test]
fn long_paths_take_the_verbatim_prefix_of_their_kind() {
    let cases = [
        ("C:\\deep\\settings.json", "\\\\?\\C:\\deep\\settings.json"),
        (
            "\\\\server\\share\\a.json",
            "\\\\?\\UNC\\server\\share\\a.json",
        ),
        ("\\\\?\\C:\\deep\\a.json", "\\\\?\\C:\\deep\\a.json"),
        ("\\\\.\\pipe\\name", "\\\\.\\pipe\\name"),
    ];
    for (full, expected) in cases {
        assert_eq!(verbatim(String::from(full)), expected);
    }
}

#[test]
fn shares_are_on_the_network_and_the_system_drive_is_not() {
    let system = system_dir().unwrap();
    let verbatim_system = format!(r"\\?\{}", system);
    let cases = [
        (r"\\192.0.2.1\share", true),
        (r"//192.0.2.1/share/project", true),
        (r"\\?\UNC\192.0.2.1\share", true),
        (r"\\?\unc\192.0.2.1\share", true),
        (system.as_str(), false),
        (verbatim_system.as_str(), false),
        (r"\project", false),
        ("project", false),
        ("", false),
    ];
    for (path, expected) in cases {
        assert_eq!(on_network(path), expected, "{}", path);
    }
}
