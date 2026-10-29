use super::ffi::{mach_host_self, Port};
use super::mount_table::{mounts, on_network};
use super::{cpu, memory};

const MACH_PORT_RIGHT_SEND: u32 = 0;
const KERN_SUCCESS: i32 = 0;
const READINGS: usize = 100;

extern "C" {
    static mach_task_self_: Port;
    fn mach_port_get_refs(task: Port, name: Port, right: u32, refs: *mut u32) -> i32;
}

fn send_references(port: Port) -> u32 {
    let mut refs = 0u32;
    let task = unsafe { mach_task_self_ };
    let got = unsafe { mach_port_get_refs(task, port, MACH_PORT_RIGHT_SEND, &mut refs) };
    assert_eq!(got, KERN_SUCCESS);
    refs
}

#[test]
fn readings_do_not_pile_up_host_port_references() {
    let port = unsafe { mach_host_self() };
    assert!(cpu().is_some() && memory().is_some());
    let before = send_references(port);
    for _ in 0..READINGS {
        assert!(cpu().is_some());
        assert!(memory().is_some());
    }
    assert_eq!(send_references(port), before);
}

#[test]
fn the_mount_table_holds_the_root_as_a_local_mount() {
    assert!(mounts().contains(&(String::from("/"), true)));
    let temp = std::env::temp_dir();
    assert!(!on_network(&temp.to_string_lossy()));
    assert!(!on_network("/Users"));
}
