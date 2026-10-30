use super::fetch::{curl, Curl, HTTP_OK};
use crate::sandbox::Sandbox;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

const BODY: &[u8] = b"0123456789";

fn serve_once() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/file", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        while reader.read_line(&mut line).unwrap_or(0) > 2 {
            line.clear();
        }
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            BODY.len()
        );
        let _ = stream.write_all(head.as_bytes());
        let _ = stream.write_all(BODY);
    });
    url
}

#[test]
fn curl_refuses_a_download_over_its_size_limit() {
    let sandbox = Sandbox::new("max-filesize");
    let dir = sandbox.dir();
    let client = Curl {
        program: curl().unwrap(),
        dir: &dir,
        protocols: "=http",
    };
    let to = sandbox.path("download");
    let fetched = client.fetch(&serve_once(), &to, 10, BODY.len());
    assert_eq!(fetched, Some(HTTP_OK));
    assert_eq!(std::fs::read(&to).unwrap(), BODY);
    std::fs::remove_file(&to).unwrap();
    assert_eq!(client.fetch(&serve_once(), &to, 10, BODY.len() - 1), None);
}
