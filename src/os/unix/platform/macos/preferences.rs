use super::super::stamp_text;
use crate::os::{home, join, read};

fn prefs() -> Option<String> {
    home().map(|h| join(&h, &["Library", "Preferences", ".GlobalPreferences.plist"]))
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

pub fn os_theme() -> Option<bool> {
    let data = read(&prefs()?)?;
    if data.starts_with(b"bplist") {
        let mut key = vec![0x5F, 0x10, 0x13];
        key.extend_from_slice(b"AppleInterfaceStyle");
        let mut dark = vec![0x54];
        dark.extend_from_slice(b"Dark");
        return Some(!(find(&data, &key).is_some() && find(&data, &dark).is_some()));
    }
    let text = String::from_utf8_lossy(&data);
    let Some(key_at) = text.find("<key>AppleInterfaceStyle</key>") else {
        return Some(true);
    };
    Some(
        !text[key_at..]
            .trim_start_matches("<key>AppleInterfaceStyle</key>")
            .trim_start()
            .starts_with("<string>Dark</string>"),
    )
}

pub fn os_stamp() -> String {
    stamp_text(prefs())
}
