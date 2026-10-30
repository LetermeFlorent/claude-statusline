const KIB: f64 = 1024.0;

fn field(meminfo: &str, name: &str) -> Option<f64> {
    let line = meminfo
        .lines()
        .find_map(|line| line.strip_prefix(name)?.strip_prefix(':'))?;
    let kib: f64 = line.trim().trim_end_matches("kB").trim().parse().ok()?;
    Some(kib * KIB)
}

const RECLAIMABLE_WITHOUT_AVAILABLE: [&str; 4] = ["MemFree", "Buffers", "Cached", "SReclaimable"];

fn available(meminfo: &str) -> Option<f64> {
    field(meminfo, "MemAvailable").or_else(|| {
        RECLAIMABLE_WITHOUT_AVAILABLE
            .iter()
            .map(|name| field(meminfo, name))
            .sum()
    })
}

pub fn parse(meminfo: &str) -> Option<(f64, f64)> {
    Some((field(meminfo, "MemTotal")?, available(meminfo)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIB: f64 = 1_048_576.0;

    #[test]
    fn available_memory_is_read_when_the_kernel_gives_it() {
        let extract =
            "MemTotal:       16384 kB\nMemFree:         1024 kB\nMemAvailable:    8192 kB\n";
        assert_eq!(parse(extract), Some((16.0 * MIB, 8.0 * MIB)));
    }

    #[test]
    fn without_mem_available_the_free_and_reclaimable_pages_add_up() {
        let extract = "MemTotal:       16384 kB\nMemFree:         1024 kB\nBuffers:          512 kB\nCached:          2048 kB\nSwapCached:      4096 kB\nSReclaimable:     256 kB\n";
        assert_eq!(parse(extract), Some((16.0 * MIB, 3.75 * MIB)));
    }

    #[test]
    fn a_missing_total_gives_nothing() {
        assert_eq!(parse("MemFree: 1024 kB\nMemAvailable: 8192 kB\n"), None);
    }
}
