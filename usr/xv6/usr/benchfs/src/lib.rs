#![no_std]
#![forbid(unsafe_code)]
#![feature(abi_x86_interrupt)]
#![feature(
    asm,
    allocator_api,
    alloc_layout_extra,
    alloc_error_handler,
    const_fn,
    const_raw_ptr_to_usize_cast,
    untagged_unions,
    panic_info_message
)]

extern crate malloc;
extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use core::panic::PanicInfo;

use usrlib::println;
use usrlib::syscalls::{sys_open, sys_fstat, sys_read, sys_write, sys_close};
use syscalls::{Syscall, Heap};
use usr::xv6::Xv6;
use usr::vfs::{DirectoryEntry, DirectoryEntryRef, INodeFileType, FileMode};

#[no_mangle]
pub fn init(s: Box<dyn Syscall + Send + Sync>, heap: Box<dyn Heap + Send + Sync>, rv6: Box<dyn Xv6>, args: &str) {
    libsyscalls::syscalls::init(s);
    rref::init(heap, libsyscalls::syscalls::sys_get_current_domain_id());
    usrlib::init(rv6.clone());
    println!("Starting rv6 benchfs with args: {}", args);

    let mut args = args.split_whitespace();
    args.next().unwrap();
    let options = args.next().unwrap_or("rw");
    let file = args.next().unwrap_or("large");
    let file_size = 128 * 1024 * 1024;

    // let buffer_sizes = [512, 1024, 4096, 8192, 16 * 1024, 256 * 1024, 1024 * 1024, 4 * 1024 * 1024, 16 * 1024 * 1024, 64 * 1024 * 1024];
    let buffer_sizes = [4 * 1024];

    for bsize in buffer_sizes.iter() {
        let bsize = *bsize;
        let mut buffer = alloc::vec![123u8; bsize];

        // 4GB
        let total_size = 4 * 1024 * 1024 * 1024;
        assert!(total_size % bsize == 0);
        if options.contains('w') {
            let fd = sys_open(file, FileMode::WRITE | FileMode::CREATE).unwrap();

            // warm up
            sys_write(fd, buffer.as_slice()).unwrap();
            rv6.sys_seek(fd, 0).unwrap();

            let mut curr_size = 0;
            let mut seek_count = 0;
            let start = rv6.sys_rdtsc();
            for offset in (bsize..total_size + bsize).step_by(bsize) {
                if offset % file_size == 0 {
                    rv6.sys_seek(fd, 0).unwrap();
                    seek_count += 1;
                }
                curr_size += sys_write(fd, buffer.as_slice()).unwrap();
            }
            let elapse = rv6.sys_rdtsc() - start;
            println!("Write: buffer size: {}, total bytes: {}, cycles: {}, seek count: {}", bsize, total_size, elapse, seek_count);
            assert_eq!(curr_size, total_size);
            
            sys_close(fd).unwrap();
        }

        // 30GB
        let total_size = 30 * 1024 * 1024 * 1024;
        assert!(total_size % bsize == 0);
        if options.contains('r') {
            let fd = sys_open(file, FileMode::READ).unwrap();

            // warm up
            rv6.sys_read(fd, buffer.as_mut_slice()).unwrap();
            rv6.sys_seek(fd, 0).unwrap();

            let mut curr_size = 0;
            let mut seek_count = 0;
            let start = rv6.sys_rdtsc();
            for offset in (bsize..total_size + bsize).step_by(bsize) {
                if offset % file_size == 0 {
                    rv6.sys_seek(fd, 0).unwrap();
                    seek_count += 1;
                }
                curr_size += rv6.sys_read(fd, buffer.as_mut_slice()).unwrap();
            }
            let elapse = rv6.sys_rdtsc() - start;
            println!("Read: buffer size: {}, total bytes: {}, cycles: {}, seek count: {}", bsize, total_size, elapse, seek_count);
            assert_eq!(curr_size, total_size);

            sys_close(fd).unwrap();
        }
    }
}


// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("benchfs panic: {:?}", info);
    libsyscalls::syscalls::sys_backtrace();
    loop {}
}
