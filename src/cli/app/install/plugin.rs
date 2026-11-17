use crate::messages;
use crate::os;
use alloc::string::String;

const FOLDER: &str = "subagent-rows";

const FILES: [(&[&str], &str); 3] = [
    (
        &[".claude-plugin", "plugin.json"],
        include_str!("../../../../plugin/subagent-rows/.claude-plugin/plugin.json"),
    ),
    (
        &["hooks", "hooks.json"],
        include_str!("../../../../plugin/subagent-rows/hooks/hooks.json"),
    ),
    (
        &["hooks", "register.ts"],
        include_str!("../../../../plugin/subagent-rows/hooks/register.ts"),
    ),
];

pub(in crate::cli::app) fn folder(exe: &str) -> Option<String> {
    os::parent(exe).map(|d| os::join(d, &[FOLDER]))
}

pub(in crate::cli::app) fn write(dir: &str) -> bool {
    FILES.iter().all(|(parts, text)| {
        let path = os::join(dir, parts);
        let made = os::parent(&path).is_some_and(os::mkdirs);
        let written = made && os::write_atomic(&path, text.as_bytes());
        if !written {
            os::err(&messages::CANNOT_WRITE.with(&[&path]));
        }
        written
    })
}
