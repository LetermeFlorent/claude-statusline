use crate::json;
use crate::num;
use crate::state::State;
use alloc::format;
use alloc::string::String;

fn positive(s: &str) -> Option<u64> {
    s.trim().parse::<u64>().ok().filter(|n| *n > 0)
}

// Fenetre d'auto-compactage : la variable d'environnement d'abord, sinon les
// reglages (dossier de config, puis projet, puis projet local), relus seulement
// quand leur taille ou leur date change.
pub fn window(
    env: Option<&str>,
    files: &[String; 3],
    state: &mut State,
    now: f64,
    stat: &dyn Fn(&str) -> Option<(u64, i64)>,
    read: &dyn Fn(&str) -> Option<String>,
) -> Option<u64> {
    if let Some(n) = env.and_then(positive) {
        return Some(n);
    }
    let cached = state.get("compact_window").and_then(|s| s.parse::<u64>().ok()).filter(|n| *n > 0);
    // Plusieurs sessions dans des dossiers differents partagent l'etat : le cache
    // ne vaut que pour le meme trio de fichiers
    let key = format!("{:016x}", fnv(files));
    let same = state.get("compact_key") == Some(key.as_str());
    if let Some(at) = state.num("compact_at").filter(|_| same) {
        if now - at < 30.0 {
            return cached;
        }
    }
    let mut stamp = String::new();
    for f in files {
        match stat(f) {
            Some((size, mtime)) => stamp.push_str(&format!("{}:{};", size, mtime)),
            None => stamp.push_str("0;"),
        }
    }
    let now_s = format!("{}", now as i64);
    if same && state.get("compact_stamp") == Some(stamp.as_str()) {
        state.set("compact_at", &now_s);
        return cached;
    }
    let mut w: Option<u64> = None;
    for f in files {
        let Some(v) = read(f).and_then(|t| json::parse(&t, true)).filter(|v| v.is_obj()) else { continue };
        if v.get("autoCompactEnabled").and_then(|x| x.bool()) == Some(false) {
            w = None;
            continue;
        }
        if let Some(n) = v.at(&["env", "CLAUDE_CODE_AUTO_COMPACT_WINDOW"]).and_then(|x| x.str()).and_then(positive) {
            w = Some(n);
            continue;
        }
        if let Some(n) = v.get("autoCompactWindow").and_then(|x| x.num()) {
            if n > 0.0 && num::is_integral(n) {
                w = Some(n as u64);
            }
        }
    }
    state.set("compact_key", &key);
    state.set("compact_stamp", &stamp);
    state.set("compact_at", &now_s);
    match w {
        Some(n) => state.set("compact_window", &format!("{}", n)),
        None => state.remove("compact_window"),
    }
    w
}

fn fnv(files: &[String; 3]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for f in files {
        for b in f.bytes().chain(core::iter::once(0)) {
            h = (h ^ b as u64).wrapping_mul(0x100000001b3);
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files() -> [String; 3] {
        [String::from("a"), String::from("b"), String::from("c")]
    }

    #[test]
    fn reglages() {
        let read = |f: &str| match f {
            "a" => Some(String::from("{\"autoCompactWindow\": 1.5e5}")),
            "b" => Some(String::from("{\"env\": {\"CLAUDE_CODE_AUTO_COMPACT_WINDOW\": \"90000\"}}")),
            _ => None,
        };
        let stat = |f: &str| if f == "c" { None } else { Some((10, 20)) };
        let mut s = State::parse("");
        assert_eq!(window(None, &files(), &mut s, 1000.0, &stat, &read), Some(90000));
        assert_eq!(s.get("compact_stamp"), Some("10:20;10:20;0;"));
        // Dans les 30 s : aucune lecture
        let none = |_: &str| -> Option<String> { panic!() };
        assert_eq!(window(None, &files(), &mut s, 1010.0, &stat, &none), Some(90000));
        // Fichiers inchanges : pas de relecture
        assert_eq!(window(None, &files(), &mut s, 2000.0, &stat, &none), Some(90000));
        assert_eq!(window(Some(" 5 "), &files(), &mut s, 2000.0, &stat, &none), Some(5));
        // Autre dossier de travail, meme dans les 30 s : relu
        let other = [String::from("a"), String::from("x"), String::from("y")];
        let only_a = |f: &str| if f == "a" { read(f) } else { None };
        assert_eq!(window(None, &other, &mut s, 2001.0, &stat, &only_a), Some(150000));
        assert_eq!(window(None, &files(), &mut s, 2002.0, &stat, &read), Some(90000));
        let off = |_: &str| Some(String::from("{\"autoCompactEnabled\": false, \"autoCompactWindow\": 5}"));
        let stat2 = |_: &str| Some((1, 1));
        assert_eq!(window(Some("1.5"), &files(), &mut s, 3000.0, &stat2, &off), None);
        assert_eq!(s.get("compact_window"), None);
    }
}
