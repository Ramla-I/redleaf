use alloc::vec::Vec;
use alloc::sync::Arc;
use core::alloc::{GlobalAlloc, Layout};
use spin::Mutex;
use crate::interrupt::{disable_irq, enable_irq};
use crate::{thread, thread::Thread};
use crate::memory::MEM_PROVIDER;
use syscalls::Heap;

// usize == *mut u8
static mut allocations: Vec<(u64, usize, Layout)> = Vec::new();
static alloc_lock: Mutex<()> = Mutex::new(());

pub struct PHeap();

impl PHeap {
    pub fn new() -> PHeap {
        PHeap {}
    }
}

impl Heap for PHeap {
    fn alloc(&self, domain_id: u64, layout: Layout) -> *mut u8 {
        disable_irq();
        let ptr = alloc_heap(domain_id, layout);
        enable_irq();
        ptr
    }

    fn dealloc(&self, domain_id: u64, ptr: *mut u8, layout: Layout) {
        disable_irq();
        dealloc_heap(domain_id, ptr, layout);
        enable_irq();
    }

    fn change_domain(&self, from_domain_id: u64, to_domain_id: u64, ptr: *mut u8, layout: Layout) {
        disable_irq();
        change_domain(from_domain_id, to_domain_id, ptr, layout);
        enable_irq();
    }

    fn get_current_domain_id(&self) -> u64 {
        disable_irq();
        let domain_id = {
            // get domain id without locking the current thread
            let thread_option: &Option<Arc<Mutex<Thread>>> = &thread::CURRENT.borrow();
            let thread_arc: &Arc<Mutex<Thread>> = thread_option.as_ref().unwrap();
            let thread_mutex: &mut Mutex<Thread> = unsafe {
                &mut *((&**thread_arc) as *const Mutex<Thread> as *mut Mutex<Thread>)
            };
            thread_mutex.get_mut().current_domain_id
        };
        enable_irq();
        domain_id
    }

    fn update_current_domain_id(&self, new_domain_id: u64) -> u64 {
        disable_irq();
        let mut old_domain_id = new_domain_id;
        {
            // swap domain id without locking the current thread
            let thread_option: &Option<Arc<Mutex<Thread>>> = &thread::CURRENT.borrow();
            let thread_arc: &Arc<Mutex<Thread>> = thread_option.as_ref().unwrap();
            let thread_mutex: &mut Mutex<Thread> = unsafe {
                &mut *((&**thread_arc) as *const Mutex<Thread> as *mut Mutex<Thread>)
            };
            let mut thread = thread_mutex.get_mut();
            core::mem::swap(&mut thread.current_domain_id, &mut old_domain_id);
        }
        enable_irq();
        old_domain_id
    }
}

fn alloc_heap(domain_id: u64, layout: Layout) -> *mut u8 {
    let ptr = unsafe { MEM_PROVIDER.alloc(layout) };
    {
        let lock = alloc_lock.lock();
        unsafe { &mut allocations }.push((domain_id, ptr as usize, layout));
        drop(lock);
    }
    ptr
}

fn dealloc_heap(domain_id: u64, ptr: *mut u8, layout: Layout) {
    let lock = alloc_lock.lock();
    unsafe { &mut allocations }.retain(|(a_domain_id, a_ptr, a_layout) | {
        // only dealloc if it's been allocated in the same way and owned by the calling domain
        if domain_id == *a_domain_id && ptr == *a_ptr as *mut u8 && layout == *a_layout {
            unsafe { MEM_PROVIDER.dealloc(ptr, layout) }
            false
        } else {
            true
        }
    });
    drop(lock);
}

fn change_domain(from_domain_id: u64, to_domain_id: u64, ptr: *mut u8, layout: Layout) {
    // TODO: this is lockless and unproved
    unsafe { &mut allocations }.iter_mut().map(|(a_domain_id, a_ptr, a_layout)| {
        if from_domain_id == *a_domain_id && ptr == *a_ptr as *mut u8 && layout == *a_layout {
            *a_domain_id = to_domain_id;
        }
    });
}

fn drop_domain(domain_id: u64) {
    let lock = alloc_lock.lock();
    unsafe { &mut allocations }.retain(|(a_domain_id, a_ptr, a_layout)| {
        if domain_id == *a_domain_id {
            unsafe { MEM_PROVIDER.dealloc(*a_ptr as *mut u8, *a_layout) }
            false
        } else {
            true
        }
    });
    drop(lock);
}
