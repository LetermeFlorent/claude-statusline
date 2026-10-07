use crate::compact;
use crate::config::{Config, Metric};
use crate::json::{self, Value};
use crate::os;
use crate::render::{self, Data, SysValue};
use crate::session::Session;
use crate::state::{self, State};
use crate::theme;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

const HELP: &str = "statusline - ligne de statut pour Claude Code

Sans argument, lit le JSON de Claude Code sur l'entree standard et ecrit la
barre sur la sortie standard.

  --install     pose l'entree statusLine dans ~/.claude/settings.json
  --uninstall   la retire
  --demo        affiche un rendu d'exemple, sans Claude Code
  --bench [n]   cout du travail seul, lancement du processus exclu
  --theme       fond detecte pour le terminal : light ou dark
  --version     numero de version
  --help        cette aide

Reglages : ~/.claude/statusline.json
";

const TEMPLATE: &str = "{
  \"bar_width\": 8,
  \"bold\": true,
  \"glyphs\": { \"filled\": \"\u{25A0}\", \"empty\": \"\u{25A0}\", \"half\": \"\u{25AA}\" },
  \"auto_compact_marker\": \"\u{AA}\",
  \"effort_style\": \"gradient\",
  \"segments\": { \"ctx\": true, \"5h\": true, \"7d\": true, \"model\": true },
  \"gradient\": { \"5h\": 0.55, \"7d\": 0.3 },
  \"system\": {
    \"enabled\": true,
    \"metrics\": [\"ram\", \"io\", \"cpu\"],
    \"gradient\": 0.45
  }
}
";

fn config_path() -> Option<String> {
    if let Some(p) = os::env("STATUSLINE_CONFIG") {
        return Some(p);
    }
    if let Some(d) = os::env("CLAUDE_CONFIG_DIR").filter(|d| !d.trim().is_empty()) {
        let p = os::join(&d, &["statusline.json"]);
        if os::exists(&p) {
            return Some(p);
        }
    }
    os::home().map(|h| os::join(&h, &[".claude", "statusline.json"]))
}

fn state_path() -> Option<String> {
    os::state_dir().map(|d| os::join(&d, &["state"]))
}

fn load_state() -> State {
    State::parse(&state_path().and_then(|p| os::read_text(&p)).unwrap_or_default())
}

fn save_state(state: &State) {
    if !state.changed() {
        return;
    }
    let Some(p) = state_path() else { return };
    if let Some(d) = os::parent(&p) {
        if !os::exists(d) {
            os::mkdirs(d);
        }
    }
    os::write_atomic(&p, state.text().as_bytes());
}

fn load_config() -> Config {
    Config::parse(config_path().and_then(|p| os::read_text(&p)).as_deref())
}

// Temps passe par etape, pour --bench : etat, config, session, rendu
struct Laps {
    t: [f64; 4],
    last: f64,
}

impl Laps {
    fn new() -> Laps {
        Laps { t: [0.0; 4], last: os::ticks() }
    }
    fn lap(&mut self, i: usize) {
        let now = os::ticks();
        self.t[i] += now - self.last;
        self.last = now;
    }
}

fn sample(cfg: &Config, state: &mut State) -> Vec<SysValue> {
    let mut out = Vec::new();
    if !cfg.sys_enabled {
        return out;
    }
    for m in &cfg.metrics {
        let v = match m {
            Metric::Ram => SysValue { metric: *m, pct: None, ram: os::memory() },
            Metric::Io => SysValue { metric: *m, pct: state.counter("io", os::disk(&cfg.disk_path)), ram: None },
            Metric::Cpu => SysValue { metric: *m, pct: state.counter("cpu", os::cpu()), ram: None },
        };
        out.push(v);
    }
    out
}

fn compact_files(cwd: Option<&str>) -> [String; 3] {
    let main = os::claude_dir().map(|d| os::join(&d, &["settings.json"])).unwrap_or_default();
    let cwd = cwd.map(String::from).or_else(os::cwd);
    let project = |f: &str| cwd.as_deref().map(|c| os::join(c, &[".claude", f])).unwrap_or_default();
    [main, project("settings.json"), project("settings.local.json")]
}

fn stat_file(p: &str) -> Option<(u64, i64)> {
    if p.is_empty() {
        None
    } else {
        os::stamp(p)
    }
}

fn read_file(p: &str) -> Option<String> {
    if p.is_empty() {
        None
    } else {
        os::read_text(p)
    }
}

fn run(input: &str, now: f64, laps: &mut Laps) -> (String, State) {
    let cfg = load_config();
    laps.lap(1);
    let s = Session::parse(input);
    laps.lap(2);
    let mut state = load_state();
    let light = theme::detect(cfg.background, &mut state);
    let quotas = state.quotas(s.live, now, &state::tag(&os::claude_dir().unwrap_or_default()));
    let window = if cfg.seg_ctx {
        let env = os::env("CLAUDE_CODE_AUTO_COMPACT_WINDOW");
        let files = compact_files(s.cwd.as_deref());
        compact::window(env.as_deref(), &files, &mut state, now, &stat_file, &read_file)
    } else {
        None
    };
    let sys = sample(&cfg, &mut state);
    laps.lap(0);
    let data = Data { ctx: s.ctx, window, quotas, model: s.model, effort: s.effort, sys };
    let out = render::render(&cfg, &cfg.colors(light), &data, now);
    laps.lap(3);
    (out, state)
}

fn demo_input(now: f64) -> String {
    let t = now as i64;
    format!(
        "{{\"model\":{{\"id\":\"claude-opus-5\",\"display_name\":\"Opus 5\"}},\"effort\":{{\"level\":\"max\"}},\
\"context_window\":{{\"context_window_size\":1000000,\"used_percentage\":34,\
\"current_usage\":{{\"input_tokens\":12000,\"cache_creation_input_tokens\":48000,\"cache_read_input_tokens\":280000}}}},\
\"rate_limits\":{{\"five_hour\":{{\"used_percentage\":36,\"resets_at\":{}}},\"seven_day\":{{\"used_percentage\":61,\"resets_at\":{}}}}}}}",
        t + 12600,
        t + 320400
    )
}

fn bench(n: usize) {
    let input = demo_input(os::now());
    let target = state_path().map(|p| format!("{}.bench", p));
    let mut laps = Laps::new();
    let mut write = 0.0;
    let mut bytes = 0;
    let start = os::ticks();
    for _ in 0..n {
        laps.last = os::ticks();
        let (out, state) = run(&input, os::now(), &mut laps);
        bytes = out.len();
        let w = os::ticks();
        if let Some(t) = &target {
            os::write_atomic(t, state.text().as_bytes());
        }
        write += os::ticks() - w;
    }
    let total = os::ticks() - start;
    if let Some(t) = &target {
        os::remove(t);
    }
    let ms = |x: f64| x * 1000.0 / n as f64;
    let mut s = format!("{} rendus, {:.3} ms par rendu ({} octets produits)\n", n, ms(total), bytes);
    for (label, v) in [("etat", laps.t[0]), ("config", laps.t[1]), ("session", laps.t[2]), ("rendu", laps.t[3]), ("ecriture", write)] {
        s.push_str(&format!("  {:<10}{:.3} ms\n", label, ms(v)));
    }
    os::out(&s);
}

fn settings_path() -> Option<String> {
    os::claude_dir().map(|d| os::join(&d, &["settings.json"]))
}

fn backup(path: &str, text: &[u8], now: f64) -> Option<String> {
    let base = format!("{}.bak-{}", path, now as i64);
    let mut name = base.clone();
    let mut i = 1;
    while os::exists(&name) {
        name = format!("{}-{}", base, i);
        i += 1;
    }
    if os::write(&name, text) {
        Some(name)
    } else {
        None
    }
}

fn fail(msg: &str) -> i32 {
    os::err(msg);
    1
}

fn install() -> i32 {
    let Some(path) = settings_path() else { return fail("dossier personnel introuvable\n") };
    let Some(exe) = os::exe() else { return fail("chemin du binaire introuvable\n") };
    let now = os::now();
    let mut v = Value::Obj(Vec::new());
    if let Some(raw) = os::read(&path) {
        let text = String::from_utf8_lossy(&raw).into_owned();
        // Un fichier illisible n'est jamais remplace : on s'arrete
        match json::parse(&text, false).filter(|x| x.is_obj()) {
            Some(x) => v = x,
            None => return fail(&format!("reglages illisibles, rien n'est modifie : {}\n", path)),
        }
        match backup(&path, &raw, now) {
            Some(b) => os::out(&format!("sauvegarde : {}\n", b)),
            None => return fail(&format!("ecriture impossible : {}.bak\n", path)),
        }
    } else if let Some(d) = os::parent(&path) {
        os::mkdirs(d);
    }
    let mut line = match v.get("statusLine") {
        Some(x) if x.is_obj() => x.clone(),
        _ => Value::Obj(Vec::new()),
    };
    line.set("type", Value::Str(String::from("command")));
    line.set("command", Value::Str(exe.replace('\\', "/")));
    if line.get("refreshInterval").is_none() {
        line.set("refreshInterval", Value::Num(3.0, String::from("3")));
    }
    v.set("statusLine", line);
    if !os::write(&path, json::pretty(&v).as_bytes()) {
        return fail(&format!("ecriture impossible : {}\n", path));
    }
    os::out(&format!("statusLine posee dans {}\n", path));
    let conf = config_path();
    if let Some(c) = conf.filter(|c| !os::exists(c)) {
        let target = os::claude_dir().map(|d| os::join(&d, &["statusline.json"])).unwrap_or(c);
        if os::write(&target, TEMPLATE.as_bytes()) {
            os::out(&format!("configuration : {}\n", target));
        }
    }
    os::out("relancez Claude Code pour voir la barre\n");
    0
}

fn uninstall() -> i32 {
    let Some(path) = settings_path() else { return fail("dossier personnel introuvable\n") };
    let raw = os::read(&path);
    let parsed = raw
        .as_ref()
        .and_then(|r| json::parse(&String::from_utf8_lossy(r), false))
        .filter(|x| x.is_obj() && x.get("statusLine").is_some());
    let (Some(raw), Some(mut v)) = (raw, parsed) else {
        os::out("aucune statusLine a retirer\n");
        return 0;
    };
    match backup(&path, &raw, os::now()) {
        Some(b) => os::out(&format!("sauvegarde : {}\n", b)),
        None => return fail(&format!("ecriture impossible : {}.bak\n", path)),
    }
    v.remove("statusLine");
    if !os::write(&path, json::pretty(&v).as_bytes()) {
        return fail(&format!("ecriture impossible : {}\n", path));
    }
    os::out(&format!("statusLine retiree de {}\n", path));
    0
}

pub fn main(args: Vec<String>) -> i32 {
    match args.get(1).map(|s| s.as_str()) {
        None => {
            let input = os::stdin().unwrap_or_default();
            let (out, state) = run(&input, os::now(), &mut Laps::new());
            os::out(&out);
            save_state(&state);
            0
        }
        Some("--demo") => {
            let now = os::now();
            let (out, _) = run(&demo_input(now), now, &mut Laps::new());
            os::out(&out);
            0
        }
        Some("--bench") => {
            let n = args.get(2).and_then(|s| s.parse::<usize>().ok()).filter(|n| *n > 0).unwrap_or(200);
            bench(n);
            0
        }
        Some("--theme") => {
            let cfg = load_config();
            let mut state = load_state();
            let light = theme::detect(cfg.background, &mut state);
            save_state(&state);
            os::out(if light { "light\n" } else { "dark\n" });
            0
        }
        Some("--install") => install(),
        Some("--uninstall") => uninstall(),
        Some("--version") | Some("-V") => {
            os::out(concat!("statusline ", env!("CARGO_PKG_VERSION"), "\n"));
            0
        }
        Some("--help") | Some("-h") => {
            os::out(HELP);
            0
        }
        Some(other) => {
            os::err(&format!("argument inconnu : {}\n{}", other, HELP));
            2
        }
    }
}
