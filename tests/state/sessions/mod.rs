mod cases;
mod shown;

use crate::common::Home;
use std::io::Write;
use std::process::Output;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SESSIONS: usize = 20;
const ROUNDS: usize = 3;
const RESET_IN_SECONDS: u64 = 3600;
const TURNED_AWAY_WITHIN: Duration = Duration::from_secs(10);
const AUTO_UPDATE: &str = "{\"auto_update\": true}";
const SHARED_KEYS: [&str; 4] = ["bg@", "compact@", "cpu_at ", "update_at "];

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

fn reset_time() -> u64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    now.as_secs() + RESET_IN_SECONDS
}
