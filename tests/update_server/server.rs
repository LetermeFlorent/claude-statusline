use super::signer::Signed;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};

pub const RELEASE_PATH: &str = "/repos/LetermeFlorent/claude-statusline/releases/latest";

fn answer(
    mut stream: TcpStream,
    release: &Signed,
    tag: &str,
    binary: &[u8],
    asked: &Sender<String>,
) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request = String::new();
    reader.read_line(&mut request).unwrap();
    let mut header = String::new();
    while reader.read_line(&mut header).unwrap() > 2 {
        header.clear();
    }
    let path = request.split_whitespace().nth(1).unwrap_or("").to_string();
    let latest = format!("{{\"tag_name\": \"{}\"}}", tag).into_bytes();
    let body = if path == RELEASE_PATH {
        &latest[..]
    } else if path.ends_with("/release-manifest") {
        &release.manifest[..]
    } else if path.ends_with("/release-manifest.sig") {
        &release.signature[..]
    } else {
        binary
    };
    let _ = asked.send(path);
    let head = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len());
    stream.write_all(head.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
}

pub fn serve(tag: String, binary: Vec<u8>, release: Signed) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (sender, asked) = channel();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            answer(stream.unwrap(), &release, &tag, &binary, &sender);
        }
    });
    (base, asked)
}
