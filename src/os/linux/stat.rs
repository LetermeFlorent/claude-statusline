use crate::os::{Activity, CPU_SOURCE};

const FIELDS_BEFORE_GUEST: usize = 8;
const IDLE: usize = 3;
const IOWAIT: usize = 4;

pub fn parse(stat: &str) -> Option<Activity> {
    let mut it = stat.lines().next()?.split_whitespace();
    if it.next()? != "cpu" {
        return None;
    }
    let jiffies: Vec<i64> = it
        .take(FIELDS_BEFORE_GUEST)
        .filter_map(|x| x.parse().ok())
        .collect();
    if jiffies.len() <= IOWAIT {
        return None;
    }
    let clock: i64 = jiffies.iter().sum();
    Some(Activity {
        source: CPU_SOURCE,
        clock,
        busy: clock - jiffies[IDLE] - jiffies[IOWAIT],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str =
        "cpu  100 20 30 400 50 6 7 8 90 10\ncpu0 50 10 15 200 25 3 3 4 45 5\nintr 12345\n";

    #[test]
    fn busy_time_leaves_out_idle_and_iowait() {
        let a = parse(FULL).unwrap();
        assert_eq!(a.clock, 621);
        assert_eq!(a.busy, 171);
        assert_eq!(a.source, CPU_SOURCE);
    }

    #[test]
    fn guest_columns_are_not_added_twice() {
        let a = parse("cpu  1 1 1 1 1 1 1 1 500 500\n").unwrap();
        assert_eq!(a.clock, 8);
    }

    #[test]
    fn an_old_kernel_with_five_columns_still_reads() {
        let a = parse("cpu  10 0 5 80 5\n").unwrap();
        assert_eq!((a.clock, a.busy), (100, 15));
    }

    #[test]
    fn four_columns_are_too_few() {
        assert!(parse("cpu  10 0 5 80\n").is_none());
    }

    #[test]
    fn the_first_line_must_be_the_total() {
        assert!(parse("cpu0 1 2 3 4 5 6 7 8\ncpu 1 2 3 4 5 6 7 8\n").is_none());
        assert!(parse("").is_none());
    }
}
