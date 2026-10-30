use alloc::string::String;

pub fn account_tag(dir: &str) -> String {
    let leaf = dir
        .trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("");
    let name: String = leaf
        .trim_start_matches('.')
        .chars()
        .map(|c| {
            let c = c.to_ascii_lowercase();
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if name.is_empty() {
        String::from("claude")
    } else {
        name
    }
}
