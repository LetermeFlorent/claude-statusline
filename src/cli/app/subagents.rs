use super::paths::load_config;
use super::persist::load_state;
use super::run::local_only;
use crate::json;
use crate::os;
use crate::render::rows::{row, Paint};
use crate::render::Terminal;
use crate::store::subagents::parse;
use crate::theme;
use alloc::format;
use alloc::string::String;

const FALLBACK_COLUMNS: usize = 80;

pub(super) fn subagents() -> i32 {
    let input = os::stdin().unwrap_or_default();
    let Ok(payload) = parse(&input) else {
        return 0;
    };
    let cfg = load_config();
    let mut state = load_state();
    let cwd = os::cwd();
    let project = local_only(cwd.as_deref());
    let light = theme::detect_light(cfg.light_override, project, &mut state, os::now());
    let colors = cfg.colors(light);
    let paint = Paint {
        cfg: &cfg,
        col: &colors,
        depth: Terminal::current().depth,
    };
    let columns = payload.columns.map_or(FALLBACK_COLUMNS, |c| c as usize);
    let mut out = String::new();
    for task in &payload.tasks {
        let content = row(&paint, task, columns);
        out.push_str(&format!(
            "{{\"id\":{},\"content\":{}}}\n",
            json::string(&task.id),
            json::string(&content)
        ));
    }
    os::out(&out);
    0
}
