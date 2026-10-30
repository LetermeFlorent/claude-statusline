use super::position::line_and_column;
use super::schema::Kind;
use super::walk::{problems, Fault};

fn faults(text: &str) -> Vec<(usize, usize, Fault)> {
    problems(text)
        .into_iter()
        .map(|problem| {
            let (line, column) = line_and_column(text, problem.at);
            (line, column, problem.fault)
        })
        .collect()
}

fn expected_paths(text: &str) -> Vec<(usize, String)> {
    faults(text)
        .into_iter()
        .filter_map(|(_, column, fault)| match fault {
            Fault::Expects(path, _) => Some((column, path)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_sample_settings_have_no_problem() {
    assert_eq!(faults(include_str!("../../statusline.json")), []);
    let loose = "// x\n{\"bar_width\": null, \"system\": {\"metrics\": [\" Dsk\"]},}";
    assert_eq!(faults(loose), []);
}

#[test]
fn each_mistake_points_to_its_line_and_column() {
    let text = "{\n  \"bar_widht\": 8,\n  \"bold\": \"no\",\n  \"system\": {\"metrics\": [\"ram\", \"gpu\"],\n    \"colors\": {\"cpu\": {\"ok\": \"#GG0000\"}}}\n}\n";
    assert_eq!(
        faults(text),
        [
            (2, 3, Fault::UnknownKey(String::from("bar_widht"))),
            (3, 11, Fault::Expects(String::from("bold"), &Kind::Flag)),
            (
                4,
                33,
                Fault::Expects(String::from("system.metrics"), &Kind::Metric)
            ),
            (
                5,
                30,
                Fault::Expects(String::from("system.colors.cpu.ok"), &Kind::Color)
            ),
        ]
    );
}

#[test]
fn a_wrong_type_is_named_by_its_full_path() {
    let text = "{\"glyphs\": \"x\", \"system\": {\"metrics\": \"ram\", \"enabled\": 1}, \"effort_style\": \"wavy\"}";
    let paths = expected_paths(text);
    let paths: Vec<(usize, &str)> = paths.iter().map(|(c, p)| (*c, p.as_str())).collect();
    assert_eq!(
        paths,
        [
            (12, "glyphs"),
            (39, "system.metrics"),
            (57, "system.enabled"),
            (77, "effort_style"),
        ]
    );
}

#[test]
fn broken_or_misplaced_json_is_reported() {
    assert_eq!(
        faults("{\n  \"bold\": tru\n}"),
        [(2, 11, Fault::InvalidJson)]
    );
    assert_eq!(faults(""), [(1, 1, Fault::InvalidJson)]);
    assert_eq!(faults("[1]"), [(1, 1, Fault::NotAnObject)]);
    assert_eq!(faults("{\"bold\": true}}"), [(1, 15, Fault::TrailingText)]);
}
