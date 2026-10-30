use super::consts::MAX_PATH;
use alloc::string::String;

pub type Handle = isize;
pub const INVALID: Handle = -1;

#[repr(C)]
#[derive(Default)]
pub struct Overlapped {
    pub internal: usize,
    pub internal_high: usize,
    pub offset: u32,
    pub offset_high: u32,
    pub event: Handle,
}

#[repr(C)]
pub struct RenameInfo {
    pub flags: u32,
    pub root: Handle,
    pub name_bytes: u32,
    pub name: [u16; 1],
}

#[repr(C)]
#[derive(Default)]
pub struct FileTime {
    pub low: u32,
    pub high: u32,
}

impl FileTime {
    pub fn ticks(&self) -> u64 {
        (self.high as u64) << 32 | self.low as u64
    }
}

#[repr(C)]
#[derive(Default)]
pub struct FileAttributeData {
    pub attributes: u32,
    pub created: FileTime,
    pub accessed: FileTime,
    pub written: FileTime,
    pub size_high: u32,
    pub size_low: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct HandleInformation {
    pub attributes: u32,
    pub created: FileTime,
    pub accessed: FileTime,
    pub written: FileTime,
    pub volume: u32,
    pub size_high: u32,
    pub size_low: u32,
    pub links: u32,
    pub index_high: u32,
    pub index_low: u32,
}

#[repr(C)]
pub struct FindData {
    pub info: FileAttributeData,
    pub reserved: [u32; 2],
    pub name: [u16; MAX_PATH as usize],
    pub short_name: [u16; 14],
}

impl FindData {
    pub fn new() -> Self {
        FindData {
            info: FileAttributeData::default(),
            reserved: [0; 2],
            name: [0; MAX_PATH as usize],
            short_name: [0; 14],
        }
    }

    pub fn name(&self) -> String {
        let len = self.name.iter().position(|&c| c == 0);
        String::from_utf16_lossy(&self.name[..len.unwrap_or(self.name.len())])
    }
}

#[repr(C)]
#[derive(Default)]
pub struct DiskPerformance {
    pub bytes_read: i64,
    pub bytes_written: i64,
    pub read_time: i64,
    pub write_time: i64,
    pub idle_time: i64,
    pub counts: [u32; 4],
    pub query_time: i64,
    pub device_number: u32,
    pub manager_name: [u16; 8],
}

#[repr(C)]
#[derive(Default)]
pub struct MemoryStatus {
    pub length: u32,
    pub load: u32,
    pub total: u64,
    pub available: u64,
    pub page_file: [u64; 2],
    pub virtual_space: [u64; 2],
    pub extended_virtual: u64,
}
