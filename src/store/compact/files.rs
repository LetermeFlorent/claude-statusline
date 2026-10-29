pub fn project_folder<'a>(
    dirs: [Option<&'a str>; 2],
    local: &dyn Fn(&str) -> bool,
    has_settings: &dyn Fn(&str) -> bool,
) -> Option<&'a str> {
    let mut usable = dirs.into_iter().flatten().filter(|dir| local(dir));
    let first = usable.next()?;
    if has_settings(first) {
        return Some(first);
    }
    Some(usable.find(|dir| has_settings(dir)).unwrap_or(first))
}
