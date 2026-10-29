mod laps;
mod sources;

use super::paths::load_config;
use super::persist::load_state;
use crate::compact;
use crate::config::Config;
use crate::os;
use crate::plain;
use crate::render::{self, Data, SysValue};
use crate::session::Session;
use crate::state::{self, State};
use crate::theme;
use crate::update_watch;
use alloc::string::String;
use alloc::vec::Vec;
pub(super) use laps::{Laps, Stage, STAGE_LABELS};
use sources::{compact_files, read_file, stat_file};
pub(super) use sources::{local_only, sample};

pub(super) type Readings<'a> = &'a dyn Fn(&Config, &mut State, f64) -> Vec<SysValue>;

pub(super) fn run(input: &str, now: f64, laps: &mut Laps) -> (String, State, bool) {
    let state = load_state();
    laps.lap(Stage::State);
    run_on(input, state, now, laps, &sample)
}

pub(super) fn run_on(
    input: &str,
    mut state: State,
    now: f64,
    laps: &mut Laps,
    readings: Readings,
) -> (String, State, bool) {
    let cfg = load_config();
    laps.lap(Stage::Config);
    let session = Session::parse(input);
    let quotas = state.quotas(
        session.live,
        now,
        &state::account_tag(&os::claude_dir().unwrap_or_default()),
    );
    laps.lap(Stage::Session);
    let project = local_only(session.project.as_deref().or(session.cwd.as_deref()));
    let light = theme::detect_light(cfg.light_override, project, &mut state, now);
    laps.lap(Stage::Theme);
    let window = if cfg.seg_ctx {
        let env = os::env("CLAUDE_CODE_AUTO_COMPACT_WINDOW");
        let files = compact_files(session.cwd.as_deref(), session.project.as_deref());
        let disable = os::env("DISABLE_AUTO_COMPACT");
        let inputs = compact::Inputs {
            env: env.as_deref(),
            disable: disable.as_deref(),
            model: session.model_id.as_deref(),
            size: session.ctx.size,
        };
        let disk = compact::Disk {
            stat: &stat_file,
            read: &read_file,
        };
        compact::window(&inputs, &files, &mut state, now, &disk)
    } else {
        None
    };
    laps.lap(Stage::Compact);
    let sys = readings(&cfg, &mut state, now);
    laps.lap(Stage::System);
    let updating = update_watch::busy();
    let data = Data {
        ctx: session.ctx,
        window,
        quotas,
        spend: session.spend.map(|spend| spend.quota()),
        cache_warm: session.cache_warm,
        model: session.model,
        effort: session.effort,
        sys,
        updating,
    };
    let out = render::render(
        &cfg,
        &cfg.colors(light),
        &data,
        now,
        &render::Terminal::current(),
    );
    let out = if plain::wanted(&os::env) {
        plain::strip(&out)
    } else {
        out
    };
    laps.lap(Stage::Render);
    (out, state, cfg.auto_update)
}
