use super::paths::load_config;
use super::persist::load_state;
use super::run::local_only;
use crate::json;
use crate::os;
use crate::render::rows::{row, Paint};
use crate::render::Terminal;
use crate::store::subagents::parse;
use crate::store::subagents::usage::Usage;
use crate::store::subagents::Used;
use crate::theme;
use alloc::format;
use alloc::string::String;

const FALLBACK_COLUMNS: usize = 80;
const CHUNK: usize = 4 << 20;

pub(super) fn subagents() -> i32 {
    let input = os::stdin().unwrap_or_default();
    let Ok(mut payload) = parse(&input) else {
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
        frame: os::now() as u64 & 2 == 0,
    };
    let project = os::claude_dir().zip(cwd.as_deref()).map(|(dir, cwd)| {
        let name: String = cwd
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        os::join(&dir, &["projects", &name])
    });
    for task in &mut payload.tasks {
        task.usage = project.as_deref().and_then(|p| usage(p, &task.id));
    }
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

/// Sums the usage of a subagent from its transcript, `<project>/<session>/subagents/agent-<id>.jsonl`,
/// reading only what was added since the last run: the totals and the offset live in the state folder.
/// shortcut: the cache file of a finished subagent is never deleted, add a cleanup when they pile up.
fn usage(project: &str, id: &str) -> Option<Used> {
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }
    let name = format!("agent-{id}.jsonl");
    let path = os::list(project, "")
        .into_iter()
        .map(|session| os::join(project, &[&session, "subagents", &name]))
        .find(|p| os::exists(p))?;
    let cache = os::state_dir().map(|d| os::join(&d, &[&format!("subagent-{id}")]));
    let usage = cache
        .as_deref()
        .and_then(os::read_text)
        .and_then(|t| Usage::decode(&t));
    // a first sight of the transcript is not a rise
    let known = usage.as_ref().map(Usage::totals);
    let mut usage = usage.unwrap_or_default();
    let before = usage.offset;
    while let Some(chunk) = os::read_from(&path, usage.offset, CHUNK) {
        let used = usage.advance(&chunk);
        // a line longer than a chunk cannot be read whole: skip it
        usage.offset += if used == 0 && chunk.len() == CHUNK {
            CHUNK
        } else {
            used
        } as u64;
        if chunk.len() < CHUNK {
            break;
        }
    }
    let now = os::now() as u64;
    if let Some(known) = known {
        usage.note_rise(known, now);
    }
    if let Some(cache) = cache.filter(|_| usage.offset != before) {
        os::write(&cache, usage.encode().as_bytes());
    }
    Some(usage.used(now))
}
