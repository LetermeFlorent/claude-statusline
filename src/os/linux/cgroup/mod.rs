use alloc::format;
use alloc::string::String;
use core::iter;

const ROOT: &str = "/sys/fs/cgroup";
const MEMBERSHIP: &str = "/proc/self/cgroup";
const UNIFIED: &str = "0::";
const LIMIT_FILE: &str = "memory.max";
const USAGE_FILE: &str = "memory.current";
const STAT_FILE: &str = "memory.stat";
const RECLAIMABLE: &str = "inactive_file ";

type Reader<'a> = dyn Fn(&str) -> Option<String> + 'a;

pub fn narrow(host: (f64, f64), read: &Reader<'_>) -> (f64, f64) {
    tightest_limit(read)
        .filter(|(limit, _)| *limit < host.0)
        .and_then(|(limit, folder)| Some((limit, limit - used(&folder, read)?.min(limit))))
        .unwrap_or(host)
}

fn tightest_limit(read: &Reader<'_>) -> Option<(f64, String)> {
    let membership = read(MEMBERSHIP)?;
    let path = membership
        .lines()
        .find_map(|line| line.strip_prefix(UNIFIED))?;
    folders(path)
        .filter_map(|folder| Some((number(&file(&folder, LIMIT_FILE, read)?)?, folder)))
        .min_by(|a, b| a.0.total_cmp(&b.0))
}

fn folders(path: &str) -> impl Iterator<Item = String> + '_ {
    let nested =
        path.split('/')
            .filter(|part| !part.is_empty())
            .scan(String::from(ROOT), |folder, part| {
                folder.push('/');
                folder.push_str(part);
                Some(folder.clone())
            });
    iter::once(String::from(ROOT)).chain(nested)
}

fn used(folder: &str, read: &Reader<'_>) -> Option<f64> {
    let current = number(&file(folder, USAGE_FILE, read)?)?;
    let reclaimable = file(folder, STAT_FILE, read)
        .and_then(|stat| {
            stat.lines()
                .find_map(|line| number(line.strip_prefix(RECLAIMABLE)?))
        })
        .unwrap_or(0.0);
    Some((current - reclaimable).max(0.0))
}

fn file(folder: &str, name: &str, read: &Reader<'_>) -> Option<String> {
    read(&format!("{}/{}", folder, name))
}

fn number(text: &str) -> Option<f64> {
    text.trim().parse::<u64>().ok().map(|n| n as f64)
}

#[cfg(test)]
mod tests;
