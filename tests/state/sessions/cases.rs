use super::shown::{assert_shows, assert_stored};
use super::{input, percent, reset_time, ROUNDS, SESSIONS, SHARED_KEYS, TURNED_AWAY_WITHIN};
use crate::common::Home;
use std::io::Write;
use std::time::Instant;

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
