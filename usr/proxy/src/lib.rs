#![no_std]

extern crate malloc;
extern crate alloc;
use rref::RRef;
use syscalls;
use libsyscalls;
use syscalls::Syscall;
use alloc::boxed::Box;
use alloc::sync::Arc;
use console::println;
use core::alloc::Layout;
use core::panic::PanicInfo;

fn get_caller_domain() -> u64 {
    libsyscalls::heap::sys_get_current_domain_id()
}

fn update_caller_domain_id(new_domain_id: u64) -> u64 {
    libsyscalls::heap::sys_update_current_domain_id(new_domain_id)
}

#[derive(Clone)]
struct Proxy {
    bdev: Arc<(Option<u64>, Option<Box<dyn usr::bdev::BDev + Send + Sync>>)>,
}

impl Proxy {
    fn new(bdev: Arc<(Option<u64>, Option<Box<dyn usr::bdev::BDev + Send + Sync>>)>) -> Proxy {
        Proxy {
            bdev
        }
    }
}

impl usr::proxy::Proxy for Proxy {
    fn proxy_clone(&self) -> Box<dyn usr::proxy::Proxy> {
        // TODO: is this safe? Box is allocated on proxy's heap
        Box::new((*self).clone())
    }

    fn bdev_new_data(&self, data: [u8; 512]) -> RRef<[u8; 512]> {
        let caller_domain = get_caller_domain();
        println!("[proxy::bdev_new_data] caller: {}", caller_domain);
        let rref = RRef::new(caller_domain, data);
        rref
    }
    fn bdev_drop_data(&self, data: RRef<[u8; 512]>) {
        RRef::drop(data);
    }

    fn bdev_read(&self, block: u32, data: &mut RRef<[u8; 512]>) {
        // TODO: Option::expect panics, instead return a Result::Err
        let callee_domain = self.bdev.0.expect("BDev interface not initialized.");
        let bdev = self.bdev.1.as_deref().expect("BDev interface not initialized.");

        // move thread to next domain
        let caller_domain = update_caller_domain_id(callee_domain);

        println!("[proxy::bdev_read] caller: {}, callee: {}", caller_domain, callee_domain);

        data.move_to(callee_domain);
        let r = bdev.read(block, data);
        data.move_to(caller_domain);

        // move thread back
        update_caller_domain_id(caller_domain);

        r
    }

    fn bdev_write(&self, block: u32, data: &[u8; 512]) {
        let callee_domain = self.bdev.0.expect("BDev interface not initialized.");
        let bdev = self.bdev.1.as_deref().expect("BDev interface not initialized.");

        // move thread to next domain
        let caller_domain = update_caller_domain_id(callee_domain);

//        data.move_to(callee_domain);
        let r = bdev.write(block, data);
//        data.move_to(caller_domain);

        // move thread back
        update_caller_domain_id(caller_domain);

        r
    }
}

#[no_mangle]
pub fn init(s: Box<dyn Syscall + Send + Sync>,
            heap: Box<dyn syscalls::Heap + Send + Sync>,
            bdev: Arc<(Option<u64>, Option<Box<dyn usr::bdev::BDev + Send + Sync>>)>) -> Box<dyn usr::proxy::Proxy + Send + Sync> {
    libsyscalls::syscalls::init(s);
    libsyscalls::heap::init(heap);

    println!("entered proxy!");

    let rref = RRef::<u64>::new(0, 10);
    println!("RRef's value: {}", *rref);
    RRef::drop(rref);
    println!("Dropped RRef");

    Box::new(Proxy::new(bdev))
}

// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("proxy panic: {:?}", info);
    libsyscalls::syscalls::sys_backtrace();
    loop {}
}
