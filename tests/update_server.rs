#![cfg(all(debug_assertions, not(target_os = "macos")))]

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::mpsc::{channel, Receiver, Sender};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BINARY: &str = env!("CARGO_BIN_EXE_statusline");
const EXE_NAME: &str = if cfg!(windows) {
    "statusline.exe"
} else {
    "statusline"
};
const RELEASE_PATH: &str = "/repos/LetermeFlorent/claude-statusline/releases/latest";
const WRONG_VERSION_EXIT: i32 = 3;
const PROXIES: [&str; 7] = [
    "http_proxy",
    "HTTP_PROXY",
    "https_proxy",
    "HTTPS_PROXY",
    "all_proxy",
    "ALL_PROXY",
    "CLAUDE_CONFIG_DIR",
];

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let root = std::env::temp_dir().join(format!("statusline-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("bin")).unwrap();
        Sandbox(root)
    }

    fn exe(&self) -> PathBuf {
        self.0.join("bin").join(EXE_NAME)
    }

    fn command(&self, arg: &str) -> Command {
        let mut command = Command::new(self.exe());
        for name in PROXIES {
            command.env_remove(name);
        }
        for name in ["HOME", "USERPROFILE", "LOCALAPPDATA", "APPDATA"] {
            command.env(name, &self.0);
        }
        command
            .env("XDG_STATE_HOME", self.0.join("state"))
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .env("NO_PROXY", "127.0.0.1")
            .arg(arg);
        command
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn newer() -> String {
    let (major, rest) = VERSION.split_once('.').unwrap();
    assert!(major.len() == 1 && major < "9", "{}", VERSION);
    format!("9.{}", rest)
}

fn relabelled(version: &str) -> Vec<u8> {
    let mut data = std::fs::read(BINARY).unwrap();
    let old = format!("statusline {}\n", VERSION).into_bytes();
    let new = format!("statusline {}\n", version).into_bytes();
    let spots: Vec<usize> = (0..data.len() - old.len())
        .filter(|&at| data[at..].starts_with(&old))
        .collect();
    assert!(!spots.is_empty());
    for at in spots {
        data[at..at + new.len()].copy_from_slice(&new);
    }
    data
}

fn answer(mut stream: TcpStream, tag: &str, binary: &[u8], asked: &Sender<String>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request = String::new();
    reader.read_line(&mut request).unwrap();
    let mut header = String::new();
    while reader.read_line(&mut header).unwrap() > 2 {
        header.clear();
    }
    let path = request.split_whitespace().nth(1).unwrap_or("").to_string();
    let release = format!("{{\"tag_name\": \"{}\"}}", tag).into_bytes();
    let body = if path == RELEASE_PATH {
        &release[..]
    } else {
        binary
    };
    let _ = asked.send(path);
    let head = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len());
    stream.write_all(head.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
}

fn serve(tag: String, binary: Vec<u8>) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (sender, asked) = channel();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            answer(stream.unwrap(), &tag, &binary, &sender);
        }
    });
    (base, asked)
}

fn update(sandbox: &Sandbox, tag: &str, binary: Vec<u8>) -> (Output, Vec<String>) {
    std::fs::copy(BINARY, sandbox.exe()).unwrap();
    let (base, asked) = serve(String::from(tag), binary);
    let output = sandbox
        .command("--update")
        .env("STATUSLINE_UPDATE_SERVER", base)
        .output()
        .unwrap();
    (output, asked.try_iter().collect())
}

fn reported(sandbox: &Sandbox) -> String {
    let output = sandbox.command("--version").output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn update_installs_the_release_of_a_local_server() {
    let sandbox = Sandbox::new("update-server");
    let version = newer();
    let tag = format!("v{}", version);
    let (output, asked) = update(&sandbox, &tag, relabelled(&version));
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(reported(&sandbox), format!("statusline {}\n", version));
    assert_eq!(asked.len(), 2);
    assert_eq!(asked[0], RELEASE_PATH);
    assert!(asked[1].contains(&format!("/releases/download/{}/", tag)));
}

#[test]
fn a_release_reporting_another_version_is_refused() {
    let sandbox = Sandbox::new("update-other-version");
    let tag = format!("v{}", newer());
    let (output, _) = update(&sandbox, &tag, std::fs::read(BINARY).unwrap());
    assert_eq!(
        output.status.code(),
        Some(WRONG_VERSION_EXIT),
        "{:?}",
        output
    );
    assert_eq!(reported(&sandbox), format!("statusline {}\n", VERSION));
    let left: Vec<String> = std::fs::read_dir(sandbox.0.join("bin"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, [EXE_NAME]);
}
