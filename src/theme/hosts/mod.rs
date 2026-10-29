use crate::json::{self, Syntax, Value};
use crate::os;
use crate::state::fingerprint;
use crate::theme::locate::places::Places;
use crate::theme::{names, Lookup, Mode, Source};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub mod color;
pub mod ghostty;
pub mod mode;
pub mod vscode;
pub mod wt;
pub mod zed;

#[derive(Clone, Copy, PartialEq)]
pub enum Host {
    Vscode,
    Wt,
    Zed,
    Ghostty,
    Other,
}

pub struct Context<'a> {
    pub places: &'a Places,
    pub order: &'a [usize],
    pub profile_id: Option<&'a str>,
    pub workspace: Option<&'a str>,
}

pub fn named(name: &str, lookup: &Lookup<'_>) -> Mode {
    names::word_mode(name)
        .or_else(|| lookup(name))
        .unwrap_or(Mode::Dark)
}

pub fn read_json(path: &str) -> Option<Value> {
    let text = os::read_text(path)?;
    json::parse(&text, Syntax::Commented).filter(|v| v.is_obj())
}

pub fn host_files(terminal: Host, places: &Places, order: &[usize]) -> Vec<String> {
    match terminal {
        Host::Vscode => places.vscode_files(order),
        Host::Wt => places.wt_files(),
        Host::Zed => places.zed_files(),
        Host::Ghostty => places.ghostty_files(),
        Host::Other => Vec::new(),
    }
}

pub fn host_source(terminal: Host) -> Source {
    match terminal {
        Host::Vscode => Source::Vscode,
        Host::Wt => Source::WindowsTerminal,
        Host::Zed => Source::Zed,
        Host::Ghostty => Source::Ghostty,
        Host::Other => Source::Assumed,
    }
}

pub fn host_stamp(terminal: Host, files: &[String], context: &Context<'_>) -> String {
    let mut stamp = String::from(match terminal {
        Host::Vscode => "vscode|",
        Host::Wt => "wt|",
        Host::Zed => "zed|",
        Host::Ghostty => "ghostty|",
        Host::Other => "os|",
    });
    stamp.push_str(context.profile_id.unwrap_or_default());
    stamp.push('|');
    for file in files.iter().map(String::as_str).chain(context.workspace) {
        let (size, mtime) = os::stamp(file).unwrap_or((0, 0));
        stamp.push_str(&format!("{}:{}:{};", file, size, mtime));
    }
    stamp.push('|');
    stamp.push_str(&os::os_stamp());
    fingerprint([stamp.as_str()])
}
