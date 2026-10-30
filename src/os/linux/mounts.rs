use crate::os::deepest_mount;
use alloc::string::String;

const MOUNT_POINT_FIELD: usize = 4;
const FIELDS_SEPARATOR: &str = " - ";
const ESCAPE_DIGITS: usize = 3;
const NETWORK_KINDS: [&str; 10] = [
    "nfs",
    "nfs4",
    "cifs",
    "smb3",
    "smbfs",
    "ceph",
    "glusterfs",
    "afs",
    "fuse.sshfs",
    "fuse.rclone",
];

struct Mount<'a> {
    kind: &'a str,
    source: &'a str,
}

fn holding<'a>(mountinfo: &'a str, path: &str) -> Option<Mount<'a>> {
    let mounts = mountinfo.lines().filter_map(|line| {
        let (mount, filesystem) = line.split_once(FIELDS_SEPARATOR)?;
        let point = unescape(mount.split_whitespace().nth(MOUNT_POINT_FIELD)?);
        let mut fields = filesystem.split_whitespace();
        let kind = fields.next()?;
        let source = fields.next()?;
        Some((point, Mount { kind, source }))
    });
    deepest_mount(mounts, path)
}

pub fn source(mountinfo: &str, path: &str) -> Option<String> {
    holding(mountinfo, path).map(|mount| unescape(mount.source))
}

pub fn on_network(mountinfo: &str, path: &str) -> bool {
    holding(mountinfo, path).is_some_and(|mount| NETWORK_KINDS.contains(&mount.kind))
}

fn unescape(field: &str) -> String {
    let mut text = String::new();
    let mut rest = field;
    while let Some(at) = rest.find('\\') {
        text.push_str(&rest[..at]);
        let code = rest
            .get(at + 1..at + 1 + ESCAPE_DIGITS)
            .filter(|digits| digits.bytes().all(|b| matches!(b, b'0'..=b'7')))
            .and_then(|digits| u8::from_str_radix(digits, 8).ok());
        let skipped = match code {
            Some(byte) => {
                text.push(char::from(byte));
                1 + ESCAPE_DIGITS
            }
            None => {
                text.push('\\');
                1
            }
        };
        rest = &rest[at + skipped..];
    }
    text.push_str(rest);
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOUNTINFO: &str = "\
22 1 0:31 /@ / rw,relatime shared:1 - btrfs /dev/nvme0n1p2 rw,ssd,subvol=/@
23 22 0:42 /@home /home rw,relatime shared:2 - btrfs /dev/sda1 rw,subvol=/@home
24 22 0:43 / /mnt/my\\040disk rw,relatime shared:3 - ext4 /dev/sdb1 rw
25 22 0:44 / /mnt/my\\040disk rw,relatime shared:4 - ext4 /dev/sdc1 rw
129 22 0:67 / /mnt/c rw,noatime - 9p C:\\134 rw,aname=drvfs;path=C:\\
140 22 0:70 / /mnt/nas rw,relatime shared:5 - nfs4 192.0.2.1:/export rw,vers=4.2
141 22 0:71 / /mnt/share rw,relatime shared:6 - cifs //192.0.2.1/share rw,vers=3.1.1
142 140 0:72 / /mnt/nas/scratch rw,relatime shared:7 - ext4 /dev/sdd1 rw
143 23 0:73 / /home/user/remote rw,nosuid shared:8 - fuse.sshfs user@192.0.2.1:/srv rw
";

    fn mount_source(path: &str) -> Option<String> {
        source(MOUNTINFO, path)
    }

    #[test]
    fn a_btrfs_path_resolves_to_the_device_of_its_mount() {
        assert_eq!(
            mount_source("/home/user/projects").as_deref(),
            Some("/dev/sda1")
        );
        assert_eq!(mount_source("/home").as_deref(), Some("/dev/sda1"));
        assert_eq!(mount_source("/").as_deref(), Some("/dev/nvme0n1p2"));
    }

    #[test]
    fn a_mount_point_only_holds_its_own_folder() {
        assert_eq!(mount_source("/homework").as_deref(), Some("/dev/nvme0n1p2"));
    }

    #[test]
    fn escaped_mount_points_match_and_the_later_mount_wins() {
        assert_eq!(
            mount_source("/mnt/my disk/notes").as_deref(),
            Some("/dev/sdc1")
        );
    }

    #[test]
    fn a_drvfs_mount_gives_its_windows_source() {
        assert_eq!(mount_source("/mnt/c/Users").as_deref(), Some("C:\\"));
    }

    #[test]
    fn folders_under_a_network_mount_are_on_the_network() {
        let cases = [
            ("/mnt/nas/project", true),
            ("/mnt/share", true),
            ("/home/user/remote/notes", true),
            ("/mnt/nas/scratch/project", false),
            ("/mnt/nasty", false),
            ("/mnt/c/Users", false),
            ("/home/user", false),
        ];
        for (path, expected) in cases {
            assert_eq!(on_network(MOUNTINFO, path), expected, "{}", path);
        }
    }
}
