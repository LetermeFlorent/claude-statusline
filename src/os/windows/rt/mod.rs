use crate::os::args;
use crate::os::windows::host::err;
use crate::os::windows::sys::Handle;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, Ordering};

#[link(name = "kernel32", kind = "raw-dylib")]
extern "system" {
    fn ExitProcess(code: u32) -> !;
    fn GetProcessHeap() -> Handle;
    fn HeapAlloc(heap: Handle, flags: u32, size: usize) -> *mut u8;
    fn HeapReAlloc(heap: Handle, flags: u32, ptr: *mut u8, size: usize) -> *mut u8;
    fn HeapFree(heap: Handle, flags: u32, ptr: *mut u8) -> i32;
}

const HEAP_ZERO_MEMORY: u32 = 8;
const MEMORY_ALLOCATION_ALIGNMENT: usize = 16;
const PANIC_EXIT_CODE: u32 = 101;
const UNWIND_EXIT_CODE: u32 = 102;

struct Heap;

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > MEMORY_ALLOCATION_ALIGNMENT {
            return null_mut();
        }
        HeapAlloc(GetProcessHeap(), 0, layout.size())
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.align() > MEMORY_ALLOCATION_ALIGNMENT {
            return null_mut();
        }
        HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, layout.size())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        HeapFree(GetProcessHeap(), 0, ptr);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.align() > MEMORY_ALLOCATION_ALIGNMENT {
            return null_mut();
        }
        HeapReAlloc(GetProcessHeap(), 0, ptr, new_size)
    }
}

#[global_allocator]
static ALLOCATOR: Heap = Heap;

struct StandardError;

impl core::fmt::Write for StandardError {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        err(s);
        Ok(())
    }
}

static PANICKING: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let first = !PANICKING.swap(true, Ordering::Relaxed);
    if let (true, Some(location)) = (first, info.location()) {
        let _ = crate::messages::PANICKED.write_to(&mut StandardError, location);
    }
    unsafe { ExitProcess(PANIC_EXIT_CODE) }
}

#[no_mangle]
pub static _fltused: i32 = 0;

#[no_mangle]
pub extern "C" fn __CxxFrameHandler3() -> ! {
    unsafe { ExitProcess(UNWIND_EXIT_CODE) }
}

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(include_str!("x86_64.s"));
#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(include_str!("aarch64.s"));

#[no_mangle]
pub extern "system" fn mainCRTStartup() -> ! {
    let code = crate::app::main(args());
    unsafe { ExitProcess(code as u32) }
}
