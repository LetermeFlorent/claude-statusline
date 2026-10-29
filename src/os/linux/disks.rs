pub type Device = (u64, u64);

const IO_MS_COLUMN: usize = 12;
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

#[cfg(test)]
mod tests {
    use super::*;

    const DISKSTATS: &str = "\
   7       0 loop0 51 0 696 9 0 0 0 0 0 40 9 0 0 0 0 0 0
 259       0 nvme0n1 7071 2245 591822 1554 6420 3651 263066 6418 0 9040 8170
 259       2 nvme0n1p2 6866 2245 584926 1522 6420 3651 263066 6418 0 8932 7940
   8       0 sda 1216 438 162410 313 0 0 0 0 0 268 313 0 0 0
 253       0 dm-0 120 0 4800 31 2 0 16 0 0 52 31
  11       0 sr0 12 0 96 5 0 0 0 0 0 20 5
";

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
