use super::*;
use crate::sandbox::Sandbox;
use crate::upgrade::sign::sha256::sha256;
use crate::upgrade::sign::sshsig::NAMESPACE;
use crate::upgrade::sign::testkit::{sign, signer_key};
use core::cell::{Cell, RefCell};
use origin::github;

pub const OLD_BINARY: &[u8] = b"old";
pub const WORKING_BINARY: &[u8] = b"statusline 99.0.0\n|demo";
const RATE_LIMITED_BODY: &[u8] = b"{\"message\": \"API rate limit exceeded\"}";
#[cfg(windows)]
pub const EXE_NAME: &str = "statusline.exe";
#[cfg(not(windows))]
pub const EXE_NAME: &str = "statusline";

fn run_fake(exe: &str, flag: &str) -> Option<String> {
    let body = std::fs::read_to_string(exe).ok()?;
    let (version, demo) = body.split_once('|')?;
    match flag {
        "--version" => Some(String::from(version)),
        "--demo" => (!demo.is_empty()).then(String::new),
        _ => None,
    }
}

pub struct Release<'a> {
    pub tag: &'a str,
    pub binary: &'a [u8],
    pub api_status: u16,
    pub manifest: Vec<u8>,
    pub signature: Vec<u8>,
    pub key: [u8; 32],
    pub asked: RefCell<Vec<String>>,
}

pub fn manifest(tag: &str, binary: &[u8]) -> Vec<u8> {
    let sum: String = sha256(binary).iter().map(|b| format!("{b:02x}")).collect();
    format!("tag {tag}\n{sum}  {}\n", asset().unwrap()).into_bytes()
}

pub fn release<'a>(tag: &'a str, binary: &'a [u8]) -> Release<'a> {
    let manifest = manifest(tag, binary);
    Release {
        tag,
        binary,
        api_status: HTTP_OK,
        signature: sign(NAMESPACE, &manifest),
        manifest,
        key: signer_key(),
        asked: RefCell::new(Vec::new()),
    }
}

impl Release<'_> {
    fn serve(&self, url: &str, to: &str, _: u32, max: usize) -> Option<u16> {
        self.asked.borrow_mut().push(String::from(url));
        let release = format!("{{\"tag_name\": \"{}\"}}", self.tag).into_bytes();
        let base = format!("/download/{}/", self.tag);
        let asset_url = format!("{base}{}", asset().unwrap());
        let (status, body) = match (url.ends_with("/releases/latest"), self.api_status) {
            (true, HTTP_OK) => (HTTP_OK, &release[..]),
            (true, limited) => (limited, RATE_LIMITED_BODY),
            _ if url.ends_with(&asset_url) => (HTTP_OK, self.binary),
            _ if url.ends_with(&format!("{base}release-manifest")) => (HTTP_OK, &self.manifest[..]),
            _ if url.ends_with(&format!("{base}release-manifest.sig")) => {
                (HTTP_OK, &self.signature[..])
            }
            _ => return None,
        };
        if body.is_empty() || body.len() > max {
            return None;
        }
        std::fs::write(to, body).ok().map(|_| status)
    }
}

pub fn failed(message: &str) -> Result<Outcome, Failure> {
    Err(Failure::from(String::from(message)))
}

pub struct Run {
    pub result: Result<Outcome, Failure>,
    pub installed: Vec<u8>,
    pub files: Vec<String>,
    pub asked: usize,
    pub probes: usize,
}

pub fn update(name: &str, release: Release) -> Run {
    let sandbox = Sandbox::new(name);
    let exe = sandbox.path(EXE_NAME);
    std::fs::write(&exe, OLD_BINARY).unwrap();
    let probes = Cell::new(0);
    let probe = |fresh: &str, flag: &str| {
        probes.set(probes.get() + 1);
        run_fake(fresh, flag)
    };
    let tools = Tools {
        origin: github(),
        fetch: &|u, t, s, m| release.serve(u, t, s, m),
        probe: &probe,
        key: release.key,
    };
    Run {
        result: attempt(&sandbox.dir(), &exe, &tools),
        installed: std::fs::read(&exe).unwrap(),
        files: sandbox.files(),
        asked: release.asked.into_inner().len(),
        probes: probes.get(),
    }
}
