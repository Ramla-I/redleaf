#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(
    asm,
    allocator_api,
    alloc_layout_extra,
    alloc_error_handler,
    const_fn,
    const_raw_ptr_to_usize_cast,
    untagged_unions,
    panic_info_message,
    get_mut_unchecked
)]

extern crate malloc;
extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::sync::Arc;
use core::panic::PanicInfo;
use libsyscalls::syscalls::{sys_create_thread, sys_yield, sys_recv_int, sys_backtrace, sys_readch_kbd};
use console::println;
use create::*;
use proxy;

#[cfg(feature = "test_guard_page")]
fn test_stack_exhaustion() -> u64 {
   
    let mut t: [u64; 4096] = [0; 4096];
    t[0] = t[1] + test_stack_exhaustion();
    t[0]
}


#[cfg(feature = "test_timer_thread")]
extern fn timer_thread() {
    println!("Registering timer thread"); 
    
    loop {
         sys_recv_int(syscalls::IRQ_TIMER);
         println!("init: got a timer interrupt"); 
    }
}


extern fn test_init_thread() {
   loop {
        println!("User init thread"); 
        sys_yield(); 
   }
}

extern fn test_init_thread2() {
   loop {
        println!("User init thread 2"); 
        sys_yield(); 
   }
}

#[cfg(feature = "test_sleep")]
fn test_sleep() {
    let start = libtime::get_ns_time();
    println!("current time {}, waiting for 10_000 ms", start);

    libtime::sys_ns_sleep(10_000_000_000); 

    let end = libtime::get_ns_time();
    println!("current time {}, waited for {} ms", end, (end - start) / 1_000_000);

}

fn test_dummy_syscall() {
    use libsyscalls::syscalls::sys_dummy;
    use libtime::get_rdtsc;

    let NUM_ITER: u64 = 20_000_000;
    let start = get_rdtsc();

    for _i in 0..NUM_ITER {
        sys_dummy();
    }

    let elapsed = get_rdtsc() - start;
    println!("Dummy syscall test: {} iterations took {} (avg: {} cycles)", NUM_ITER,
                        elapsed, elapsed / NUM_ITER);
}

// AB: XXX: The following is is not supported in Rust at the moment
//
//pub fn init(s: Box<dyn syscalls::Syscall 
//                    + create::CreateXv6 + create::CreateXv6FS /* + CreateXv6User */
//                    + create::CreatePCI + create::CreateAHCI + Send + Sync>) 
// See
//   rustc --explain E0225
//
// We have to re-write in an ugly way
#[no_mangle]
pub fn init(s: Box<dyn syscalls::Syscall + Send + Sync>,
            ints: Box<dyn syscalls::Interrupt + Send + Sync>,
            create_proxy: Box<dyn proxy::CreateProxy>,
            create_xv6: Arc<dyn create::CreateXv6>,
            create_xv6fs: Arc<dyn create::CreateXv6FS>,
            create_xv6usr: Arc<dyn create::CreateXv6Usr + Send + Sync>,
            create_pci: Arc<dyn create::CreatePCI>,
            create_ixgbe: Arc<dyn create::CreateIxgbe>,
            create_benchnet: Arc<dyn create::CreateBenchnet>,
            create_ahci: Arc<dyn create::CreateAHCI>,
            create_membdev: Arc<dyn create::CreateMemBDev>,
            create_bdev_shadow: Arc<dyn create::CreateBDevShadow>,
            create_dom_a: Arc<dyn create::CreateDomA>,
            create_dom_b: Arc<dyn create::CreateDomB>,
            create_dom_c: Arc<dyn create::CreateDomC>,
            create_dom_d: Arc<dyn create::CreateDomD>,
            create_shadow: Arc<dyn create::CreateShadow>) {
    libsyscalls::syscalls::init(s);

    let ints_clone = ints.int_clone(); 
    libsyscalls::syscalls::init_interrupts(ints);

    //let b = Box::new(4);
    //let r = sys_alloc();
    let mut v1: Vec<u64> = Vec::with_capacity(1024);
    for i in 0..2048 {
        v1.push(i);
    }

    println!("{} {} {}", "init", "userland", 1);

    //println!("init userland print works");


    #[cfg(feature = "test_guard_page")]
    {
        println!("start the test guard page test"); 
        let foo =  test_stack_exhaustion();
        println!("test guard page: {}", foo); 
    }

    #[cfg(feature = "test_timer_thread")]
    {
        let t = sys_create_thread("init_int[timer]", timer_thread); 
        t.set_priority(10);
    }

    #[cfg(feature = "test_sleep")]
    test_sleep();

    
    #[cfg(feature = "test_threads")]
    {
        let t = sys_create_thread("init_thread", test_init_thread); 
        t.set_affinity(1); 

        let t2 = sys_create_thread("init_thread_2", test_init_thread2); 
        t2.set_affinity(0); 

        #[cfg(feature = "test_sleep")]
        test_sleep();

        println!("Setting affinity to CPUs 2 and 3"); 
        t.set_affinity(2); 
        t2.set_affinity(3); 

         #[cfg(feature = "test_sleep")]
        test_sleep();

        println!("Setting affinity to CPUs 1 and 1"); 
        t.set_affinity(1); 
        t2.set_affinity(1); 

        drop(t); 
        drop(t2); 
    }

    // test_dummy_syscall();

    println!("about to create proxy");
    let (dom_proxy, proxy) = create_proxy.create_domain_proxy(
        create_pci,
        create_ahci,
        create_membdev,
        create_bdev_shadow,
        create_ixgbe,
        create_benchnet,
        create_xv6fs,
        create_xv6usr,
        create_xv6,
        create_dom_a,
        create_dom_b,
        create_dom_c,
        create_dom_d,
        create_shadow,
    );
    println!("created proxy");


    println!("Creating pci");
    let (dom_pci, pci) = proxy.as_create_pci().create_domain_pci();

    let pci2 = pci.pci_clone();

    #[cfg(not(feature = "membdev"))]
    let (dom_ahci, bdev) = proxy.as_create_ahci().create_domain_ahci(pci);

    #[cfg(feature = "membdev")]
    // let (dom_ahci, bdev) = proxy.as_create_membdev().create_domain_membdev();
    let (dom_ahci, bdev) = proxy.as_create_bdev_shadow().create_domain_bdev_shadow(proxy.as_create_membdev());

    println!("Creating ixgbe");
    let (dom_ixgbe, net) = proxy.as_create_ixgbe().create_domain_ixgbe(pci2);

    #[cfg(feature = "benchnet")]
    let _ = proxy.as_create_benchnet().create_domain_benchnet(net);

    #[cfg(feature = "test_ab")]
    {
        let (dom_dom_a, dom_a) = proxy.as_create_dom_a().create_domain_dom_a();
        let dom_dom_b = proxy.as_create_dom_b().create_domain_dom_b(dom_a);
    }

    #[cfg(feature = "test_shadow")]
    {
        //let (dom_dom_c, dom_c) = proxy.as_create_dom_c().create_domain_dom_c();
        let (dom_shadow, shadow) = proxy.as_create_shadow().create_domain_shadow(proxy.as_create_dom_c());
        let dom_dom_d = proxy.as_create_dom_d().create_domain_dom_d(shadow);
    }

    #[cfg(not(feature = "benchnet"))]
    {
        let (dom_xv6, rv6) = proxy.as_create_xv6().create_domain_xv6kernel(ints_clone, proxy.as_create_xv6fs(), proxy.as_create_xv6usr(), bdev, net);
        rv6.sys_spawn_domain("/init", "/init", array_init::array_init(|_| None)).unwrap();
    }
}

// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sys_backtrace();
    loop {}
}
