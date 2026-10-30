#![cfg(windows)]

const EXPECTED: [&str; 1] = ["kernel32.dll"];
const PE_HEADER_POINTER: usize = 0x3c;
const SECTION_COUNT: usize = 6;
const OPTIONAL_HEADER_SIZE: usize = 20;
const OPTIONAL_HEADER: usize = 24;
const IMPORT_DIRECTORY: usize = 120;
const SECTION_HEADER_SIZE: usize = 40;
const SECTION_VIRTUAL_SIZE: usize = 8;
const SECTION_START: usize = 12;
const SECTION_RAW_SIZE: usize = 16;
const SECTION_RAW_START: usize = 20;
const IMPORT_DESCRIPTOR_SIZE: usize = 20;
const IMPORT_NAME: usize = 12;

struct Section {
    start: usize,
    size: usize,
    raw_start: usize,
}

struct Image {
    data: Vec<u8>,
    sections: Vec<Section>,
}

fn half(data: &[u8], at: usize) -> usize {
    usize::from(u16::from_le_bytes([data[at], data[at + 1]]))
}

fn full(data: &[u8], at: usize) -> usize {
    u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]) as usize
}

impl Image {
    fn read(path: &str) -> Image {
        let data = std::fs::read(path).unwrap();
        let header = full(&data, PE_HEADER_POINTER);
        let optional = header + OPTIONAL_HEADER;
        let first = optional + half(&data, header + OPTIONAL_HEADER_SIZE);
        let sections = (0..half(&data, header + SECTION_COUNT))
            .map(|index| first + index * SECTION_HEADER_SIZE)
            .map(|at| Section {
                start: full(&data, at + SECTION_START),
                size: full(&data, at + SECTION_VIRTUAL_SIZE)
                    .max(full(&data, at + SECTION_RAW_SIZE)),
                raw_start: full(&data, at + SECTION_RAW_START),
            })
            .collect();
        Image { data, sections }
    }

    fn offset(&self, address: usize) -> usize {
        let section = self
            .sections
            .iter()
            .find(|section| (section.start..section.start + section.size).contains(&address))
            .unwrap();
        address - section.start + section.raw_start
    }

    fn text(&self, address: usize) -> String {
        let start = self.offset(address);
        let length = self.data[start..]
            .iter()
            .position(|&byte| byte == 0)
            .unwrap();
        String::from_utf8_lossy(&self.data[start..start + length]).to_lowercase()
    }

    fn libraries(&self) -> Vec<String> {
        let header = full(&self.data, PE_HEADER_POINTER);
        let table = self.offset(full(
            &self.data,
            header + OPTIONAL_HEADER + IMPORT_DIRECTORY,
        ));
        (0..)
            .map(|index| {
                full(
                    &self.data,
                    table + index * IMPORT_DESCRIPTOR_SIZE + IMPORT_NAME,
                )
            })
            .take_while(|&name| name != 0)
            .map(|name| self.text(name))
            .collect()
    }
}

#[test]
fn the_program_loads_nothing_but_kernel32() {
    let image = Image::read(env!("CARGO_BIN_EXE_statusline"));
    assert_eq!(image.libraries(), EXPECTED);
}
