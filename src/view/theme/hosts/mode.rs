use super::ghostty::ghostty;
use super::vscode::vscode;
use super::wt::{wt, wt_settings};
use super::zed::zed;
use super::{read_json, Context, Host};
use crate::json::Value;
use crate::os;
use crate::theme::{lookup, Mode};
use alloc::string::String;
use alloc::vec::Vec;

pub fn host_mode(terminal: Host, files: &[String], context: &Context<'_>) -> Mode {
    let places = context.places;
    let fallback = if places.unresolved {
        Mode::System
    } else {
        Mode::Dark
    };
    let first_json = || files.iter().find_map(|file| read_json(file));
    match terminal {
        Host::Vscode => {
            let layers: Vec<Value> = context
                .workspace
                .and_then(read_json)
                .into_iter()
                .chain(first_json())
                .collect();
            if layers.is_empty() {
                return fallback;
            }
            let roots = places.vscode_extensions(context.order);
            vscode(&layers, &|name| lookup::vscode_ui_theme(&roots, name))
        }
        Host::Wt => {
            let os_light = || os::os_theme().unwrap_or(false);
            wt_settings(files, context.profile_id).map_or(fallback, |settings| {
                wt(&settings, context.profile_id, &os_light)
            })
        }
        Host::Zed => first_json().map_or(Mode::System, |settings| {
            zed(&settings, &|name| {
                lookup::zed_appearance(&places.zed_theme_dirs(), name)
            })
        }),
        Host::Ghostty => {
            let texts: Vec<String> = files
                .iter()
                .filter_map(|file| os::read_text(file))
                .collect();
            let dirs = places.ghostty_theme_dirs();
            ghostty(&texts.join("\n"), &|name| {
                lookup::ghostty_theme(&dirs, name)
            })
        }
        Host::Other => other_host_mode(),
    }
}

fn other_host_mode() -> Mode {
    if cfg!(windows) {
        Mode::Dark
    } else {
        Mode::System
    }
}
