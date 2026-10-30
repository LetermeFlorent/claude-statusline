use super::*;

const MIB: f64 = 1_048_576.0;
const HOST: (f64, f64) = (16384.0 * MIB, 6144.0 * MIB);

fn narrowed(files: &[(&str, &str)]) -> (f64, f64) {
    narrow(HOST, &|path| {
        files
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, text)| String::from(*text))
    })
}

#[test]
fn a_container_limit_replaces_the_host_memory() {
    let memory = narrowed(&[
        ("/proc/self/cgroup", "0::/\n"),
        ("/sys/fs/cgroup/memory.max", "536870912\n"),
        ("/sys/fs/cgroup/memory.current", "314572800\n"),
        (
            "/sys/fs/cgroup/memory.stat",
            "anon 1000\ninactive_file 104857600\n",
        ),
    ]);
    assert_eq!(memory, (512.0 * MIB, 312.0 * MIB));
}

#[test]
fn no_limit_keeps_the_host() {
    let memory = narrowed(&[
        ("/proc/self/cgroup", "0::/user.slice/session-2.scope\n"),
        ("/sys/fs/cgroup/user.slice/memory.max", "max\n"),
        (
            "/sys/fs/cgroup/user.slice/session-2.scope/memory.max",
            "max\n",
        ),
        (
            "/sys/fs/cgroup/user.slice/session-2.scope/memory.current",
            "104857600\n",
        ),
    ]);
    assert_eq!(memory, HOST);
}

#[test]
fn the_strictest_folder_on_the_way_down_wins() {
    let memory = narrowed(&[
        ("/proc/self/cgroup", "0::/user.slice/app.scope\n"),
        ("/sys/fs/cgroup/user.slice/memory.max", "1073741824\n"),
        ("/sys/fs/cgroup/user.slice/memory.current", "524288000\n"),
        (
            "/sys/fs/cgroup/user.slice/app.scope/memory.max",
            "2147483648\n",
        ),
        (
            "/sys/fs/cgroup/user.slice/app.scope/memory.current",
            "104857600\n",
        ),
    ]);
    assert_eq!(memory, (1024.0 * MIB, 524.0 * MIB));
}

#[test]
fn a_limit_above_the_host_keeps_the_host() {
    let memory = narrowed(&[
        ("/proc/self/cgroup", "0::/\n"),
        ("/sys/fs/cgroup/memory.max", "68719476736\n"),
        ("/sys/fs/cgroup/memory.current", "104857600\n"),
    ]);
    assert_eq!(memory, HOST);
}

#[test]
fn a_cgroup_v1_membership_keeps_the_host() {
    let memory = narrowed(&[
        (
            "/proc/self/cgroup",
            "12:memory:/docker/4f2a\n1:name=systemd:/docker/4f2a\n",
        ),
        ("/sys/fs/cgroup/memory.max", "536870912\n"),
        ("/sys/fs/cgroup/memory.current", "104857600\n"),
    ]);
    assert_eq!(memory, HOST);
}

#[test]
fn usage_over_the_limit_reads_as_full() {
    let memory = narrowed(&[
        ("/proc/self/cgroup", "0::/\n"),
        ("/sys/fs/cgroup/memory.max", "536870912\n"),
        ("/sys/fs/cgroup/memory.current", "600000000\n"),
    ]);
    assert_eq!(memory, (512.0 * MIB, 0.0));
}

#[test]
fn an_unknown_usage_keeps_the_host() {
    let memory = narrowed(&[
        ("/proc/self/cgroup", "0::/\n"),
        ("/sys/fs/cgroup/memory.max", "536870912\n"),
    ]);
    assert_eq!(memory, HOST);
}
