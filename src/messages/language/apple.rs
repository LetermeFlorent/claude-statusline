use super::{from_tag, Language};
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(target_os = "macos")]
mod preferences {
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::ffi::{c_char, c_void};

    const UTF8: u32 = 0x0800_0100;
    const TAG_BYTES: usize = 32;
    const MAX_TAGS: isize = 16;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFLocaleCopyPreferredLanguages() -> *const c_void;
        fn CFArrayGetCount(array: *const c_void) -> isize;
        fn CFArrayGetValueAtIndex(array: *const c_void, index: isize) -> *const c_void;
        fn CFStringGetCString(s: *const c_void, buf: *mut c_char, size: isize, enc: u32) -> u8;
        fn CFRelease(cf: *const c_void);
    }

    pub fn tags() -> Vec<String> {
        let mut tags = Vec::new();
        unsafe {
            let list = CFLocaleCopyPreferredLanguages();
            if list.is_null() {
                return tags;
            }
            for index in 0..CFArrayGetCount(list).min(MAX_TAGS) {
                let mut buffer = [0 as c_char; TAG_BYTES];
                let item = CFArrayGetValueAtIndex(list, index);
                if CFStringGetCString(item, buffer.as_mut_ptr(), TAG_BYTES as isize, UTF8) != 0 {
                    let bytes: Vec<u8> = buffer
                        .iter()
                        .take_while(|b| **b != 0)
                        .map(|b| *b as u8)
                        .collect();
                    tags.push(String::from_utf8_lossy(&bytes).into_owned());
                }
            }
            CFRelease(list);
        }
        tags
    }
}

#[cfg(target_os = "macos")]
fn read() -> Vec<String> {
    preferences::tags()
}

#[cfg(not(target_os = "macos"))]
fn read() -> Vec<String> {
    Vec::new()
}

pub fn preferred() -> Option<Language> {
    read().iter().find_map(|tag| from_tag(tag))
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn the_system_lists_at_least_one_preferred_language_tag() {
        let tags = preferences::tags();
        assert!(!tags.is_empty());
        for tag in &tags {
            assert!(tag.is_ascii() && !tag.is_empty(), "{}", tag);
        }
    }

    #[test]
    fn the_first_supported_preferred_language_is_found() {
        assert!(preferred().is_some());
    }
}
