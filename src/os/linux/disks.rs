use alloc::string::String;

pub type Device = (u64, u64);

const IO_MS_COLUMN: usize = 12;
const MOUNT_POINT_FIELD: usize = 4;
const SOURCE_FIELD: usize = 1;
const FIELDS_SEPARATOR: &str = " - ";
const ESCAPE_DIGITS: usize = 3;
const REMOVABLE_DRIVES: [&str; 2] = ["sr", "fd"];

struct Row<'a> {
    device: Device,
    name: &'a str,
    io_ms: i64,
}

pub fn io_ms(diskstats: &str, device: Device) -> Option<i64> {
    rows(diskstats)
        .find(|row| row.device == device)
        .map(|row| row.io_ms)
}

pub fn sole_disk(diskstats: &str, physical: &dyn Fn(&str) -> bool) -> Option<Device> {
    let mut disks = rows(diskstats)
        .filter(|row| {
            !REMOVABLE_DRIVES
                .iter()
                .any(|kind| row.name.starts_with(kind))
        })
        .filter(|row| physical(row.name));
    let only = disks.next()?;
    disks.next().is_none().then_some(only.device)
}

pub fn mount_source(mountinfo: &str, path: &str) -> Option<String> {
    mountinfo
        .lines()
        .filter_map(|line| {
            let (mount, filesystem) = line.split_once(FIELDS_SEPARATOR)?;
            let point = unescape(mount.split_whitespace().nth(MOUNT_POINT_FIELD)?);
            let source = filesystem.split_whitespace().nth(SOURCE_FIELD)?;
            holds(&point, path).then_some((point.len(), source))
        })
        .max_by_key(|(depth, _)| *depth)
        .map(|(_, source)| unescape(source))
}

fn holds(point: &str, path: &str) -> bool {
    point == "/"
        || path
            .strip_prefix(point)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

fn rows(diskstats: &str) -> impl Iterator<Item = Row<'_>> {
    diskstats.lines().filter_map(|line| {
        let mut columns = line.split_whitespace();
        let device = (columns.next()?.parse().ok()?, columns.next()?.parse().ok()?);
        let name = columns.next()?;
        let io_ms = line.split_whitespace().nth(IO_MS_COLUMN)?.parse().ok()?;
        Some(Row {
            device,
            name,
            io_ms,
        })
    })
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
";

    const DISKSTATS: &str = "\
   7       0 loop0 51 0 696 9 0 0 0 0 0 40 9 0 0 0 0 0 0
 259       0 nvme0n1 7071 2245 591822 1554 6420 3651 263066 6418 0 9040 8170
 259       2 nvme0n1p2 6866 2245 584926 1522 6420 3651 263066 6418 0 8932 7940
   8       0 sda 1216 438 162410 313 0 0 0 0 0 268 313 0 0 0
 253       0 dm-0 120 0 4800 31 2 0 16 0 0 52 31
  11       0 sr0 12 0 96 5 0 0 0 0 0 20 5
";

    fn source(path: &str) -> Option<String> {
        mount_source(MOUNTINFO, path)
    }

    #[test]
    fn a_btrfs_path_resolves_to_the_device_of_its_mount() {
        assert_eq!(source("/home/user/projects").as_deref(), Some("/dev/sda1"));
        assert_eq!(source("/home").as_deref(), Some("/dev/sda1"));
        assert_eq!(source("/").as_deref(), Some("/dev/nvme0n1p2"));
    }

    #[test]
    fn a_mount_point_only_holds_its_own_folder() {
        assert_eq!(source("/homework").as_deref(), Some("/dev/nvme0n1p2"));
    }

    #[test]
    fn escaped_mount_points_match_and_the_later_mount_wins() {
        assert_eq!(source("/mnt/my disk/notes").as_deref(), Some("/dev/sdc1"));
    }

    #[test]
    fn a_drvfs_mount_gives_its_windows_source() {
        assert_eq!(source("/mnt/c/Users").as_deref(), Some("C:\\"));
    }

    #[test]
    fn io_time_comes_from_the_line_of_the_device() {
        assert_eq!(io_ms(DISKSTATS, (259, 2)), Some(8932));
        assert_eq!(io_ms(DISKSTATS, (8, 0)), Some(268));
        assert_eq!(io_ms(DISKSTATS, (8, 16)), None);
    }

    #[test]
    fn the_only_physical_disk_is_the_fallback() {
        let physical = |name: &str| ["nvme0n1", "sr0"].contains(&name);
        assert_eq!(sole_disk(DISKSTATS, &physical), Some((259, 0)));
    }

    #[test]
    fn several_physical_disks_give_no_fallback() {
        let physical = |name: &str| ["nvme0n1", "sda"].contains(&name);
        assert_eq!(sole_disk(DISKSTATS, &physical), None);
    }
}
