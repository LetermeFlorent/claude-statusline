use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const SESSIONS: usize = 20;
const ROUNDS: usize = 3;
const RESET_IN_SECONDS: u64 = 3600;
const INHERITED: [&str; 9] = [
    "CLAUDE_CONFIG_DIR",
    "CLAUDE_CODE_AUTO_COMPACT_WINDOW",
    "CLAUDE_AUTOCOMPACT_PCT_OVERRIDE",
    "STATUSLINE_CONFIG",
    "STATUSLINE_BG",
    "STATUSLINE_LANG",
    "XDG_STATE_HOME",
    "XDG_CONFIG_HOME",
    "NoDefaultCurrentDirectoryInExePath",
];

struct Home(PathBuf);

impl Home {
    fn new() -> Home {
        let root = std::env::temp_dir().join(format!("statusline-sessions-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Home(root)
    }

    fn state_dir(&self) -> PathBuf {
        let base = if cfg!(windows) {
            self.0.join("local")
        } else if cfg!(target_os = "macos") {
            self.0.join("Library").join("Application Support")
        } else {
            self.0.join("state")
        };
        base.join("claude-statusline")
    }

    fn session(&self, account: usize) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_statusline"));
        for name in INHERITED {
            command.env_remove(name);
        }
        command
            .env("HOME", &self.0)
            .env("USERPROFILE", &self.0)
            .env("LOCALAPPDATA", self.0.join("local"))
            .env("APPDATA", self.0.join("roaming"))
            .env("XDG_STATE_HOME", self.0.join("state"))
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .env(
                "CLAUDE_CONFIG_DIR",
                self.0.join(format!(".claude-a{}", account)),
            )
            .env("NO_COLOR", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    fn refresh_all_at_once(&self, input: &dyn Fn(usize) -> String) -> Vec<Output> {
        let mut children: Vec<_> = (0..SESSIONS)
            .map(|account| self.session(account).spawn().unwrap())
            .collect();
        for (account, child) in children.iter_mut().enumerate() {
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(input(account).as_bytes()).unwrap();
        }
        children
            .into_iter()
            .map(|child| child.wait_with_output().unwrap())
            .collect()
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn percent(account: usize, round: usize) -> usize {
    11 + account + round * SESSIONS
}

fn input(rate_limits: Option<(usize, u64)>) -> String {
    let limits = rate_limits.map_or(String::new(), |(pct, resets)| {
        format!(
            ",\"rate_limits\":{{\"five_hour\":{{\"used_percentage\":{},\"resets_at\":{}}}}}",
            pct, resets
        )
    });
    format!(
        "{{\"model\":{{\"display_name\":\"Opus\"}},\"context_window\":{{\"context_window_size\":200000,\"used_percentage\":30}}{}}}",
        limits
    )
}

fn five_hour_percent(output: &Output) -> Option<String> {
    let text = String::from_utf8_lossy(&output.stdout);
    let segment = text.split(" | ").find(|part| part.starts_with("5h "))?;
    segment
        .split(' ')
        .find(|word| word.ends_with('%'))
        .map(String::from)
}

fn assert_shows(outputs: &[Output], round: usize) {
    for (account, output) in outputs.iter().enumerate() {
        let shown = String::from_utf8_lossy(&output.stdout);
        let errors = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "account {}: {}", account, errors);
        assert!(errors.is_empty(), "account {}: {}", account, errors);
        let expected = format!("{}%", percent(account, round));
        assert_eq!(
            five_hour_percent(output).as_deref(),
            Some(expected.as_str()),
            "account {}: {}",
            account,
            shown
        );
    }
}

#[test]
fn sessions_refreshing_at_once_keep_every_account_quota() {
    let home = Home::new();
    let resets = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + RESET_IN_SECONDS;
    for round in 0..ROUNDS {
        let live =
            home.refresh_all_at_once(&|account| input(Some((percent(account, round), resets))));
        assert_shows(&live, round);
    }
    let last = ROUNDS - 1;
    assert_shows(&home.refresh_all_at_once(&|_| input(None)), last);
    let state = std::fs::read_to_string(home.state_dir().join("state")).unwrap();
    assert!(
        state
            .lines()
            .all(|line| line.split_once(' ').is_some_and(|(key, _)| !key.is_empty())),
        "{}",
        state
    );
    for account in 0..SESSIONS {
        let line = format!("q5_pct@claude-a{} {}", account, percent(account, last));
        assert!(
            state.lines().any(|kept| kept == line),
            "{} lost:\n{}",
            line,
            state
        );
    }
    let leftovers: Vec<String> = std::fs::read_dir(home.state_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{:?}", leftovers);
}
