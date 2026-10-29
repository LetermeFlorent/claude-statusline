mod common;
mod state_folder;

use common::Home;
use std::io::Write;
use std::process::Output;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SESSIONS: usize = 20;
const ROUNDS: usize = 3;
const RESET_IN_SECONDS: u64 = 3600;
const TURNED_AWAY_WITHIN: Duration = Duration::from_secs(10);
const AUTO_UPDATE: &str = "{\"auto_update\": true}";
const SHARED_KEYS: [&str; 5] = ["bg_host ", "bg_stamp ", "compact@", "cpu_at ", "update_at "];

impl Home {
    fn check_updates_in_every_account(&self) {
        for account in 0..SESSIONS {
            let folder = self.0.join(format!(".claude-a{}", account));
            std::fs::create_dir_all(&folder).unwrap();
            std::fs::write(folder.join("statusline.json"), AUTO_UPDATE).unwrap();
        }
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

fn whole_bar(shown: &str) -> bool {
    let lines: Vec<&str> = shown.split_terminator('\n').collect();
    shown.ends_with('\n')
        && lines.len() == 2
        && lines[0].starts_with("ctx ")
        && lines[0].ends_with(" | Opus")
        && lines[1].starts_with("ram ")
        && lines[1].matches(" | ").count() == 2
}

fn assert_shows(outputs: &[Output], round: usize) {
    for (account, output) in outputs.iter().enumerate() {
        let shown = String::from_utf8_lossy(&output.stdout);
        let errors = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "account {}: {}", account, errors);
        assert!(errors.is_empty(), "account {}: {}", account, errors);
        assert!(whole_bar(&shown), "account {}: {:?}", account, shown);
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

fn reset_time() -> u64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    now.as_secs() + RESET_IN_SECONDS
}

fn assert_stored(home: &Home, round: usize) -> String {
    let state = std::fs::read_to_string(home.state_dir().join("state")).unwrap();
    assert!(
        state
            .lines()
            .all(|line| line.split_once(' ').is_some_and(|(key, _)| !key.is_empty())),
        "{}",
        state
    );
    for account in 0..SESSIONS {
        let line = format!("q5_pct@claude-a{} {}", account, percent(account, round));
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
        .filter(|name| name.ends_with(".tmp") || name.ends_with(".changes"))
        .collect();
    assert!(leftovers.is_empty(), "{:?}", leftovers);
    state
}

#[test]
fn sessions_refreshing_at_once_keep_every_account_quota() {
    let home = Home::new("sessions");
    home.check_updates_in_every_account();
    let resets = reset_time();
    for round in 0..ROUNDS {
        let live =
            home.refresh_all_at_once(&|account| input(Some((percent(account, round), resets))));
        assert_shows(&live, round);
    }
    let last = ROUNDS - 1;
    assert_shows(&home.refresh_all_at_once(&|_| input(None)), last);
    let state = assert_stored(&home, last);
    for key in SHARED_KEYS {
        let kept = state.lines().any(|line| line.starts_with(key));
        assert!(kept, "{} lost:\n{}", key, state);
    }
}

#[test]
fn sessions_turned_away_by_a_held_lock_lose_no_quota() {
    let home = Home::new("held-lock");
    std::fs::create_dir_all(home.state_dir()).unwrap();
    let holder = std::fs::File::create(home.state_dir().join("state.lock")).unwrap();
    holder.lock().unwrap();
    let resets = reset_time();
    let started = Instant::now();
    let turned_away =
        home.refresh_all_at_once(&|account| input(Some((percent(account, 0), resets))));
    let waited = started.elapsed();
    drop(holder);
    assert!(waited < TURNED_AWAY_WITHIN, "{:?}", waited);
    assert_shows(&turned_away, 0);
    assert_shows(&home.refresh_all_at_once(&|_| input(None)), 0);
    let mut late = home.session(SESSIONS).spawn().unwrap();
    let late_input = input(Some((percent(SESSIONS, 0), resets)));
    let mut stdin = late.stdin.take().unwrap();
    stdin.write_all(late_input.as_bytes()).unwrap();
    drop(stdin);
    assert!(late.wait_with_output().unwrap().status.success());
    assert_stored(&home, 0);
}
