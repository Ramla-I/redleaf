// AB: for now lets use a global lock, we'll get rid of it later
//pub static CONTEXT_SWITCH_LOCK: AtomicBool = AtomicBool::new(false);

use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use core::cell::RefCell;
use alloc::rc::Rc;
use crate::halt;
use crate::interrupt::{disable_irq, enable_irq};
use spin::Mutex;
use alloc::sync::Arc; 

const MAX_PRIO: usize = 15;
const MAX_CPUS: usize = 64;

/// Per-CPU scheduler
#[thread_local]
static SCHED: RefCell<Scheduler> = RefCell::new(Scheduler::new()); 

/// Per-CPU current thread
#[thread_local]
static CURRENT: RefCell<Option<Rc<RefCell<Thread>>>> = RefCell::new(None); 

static REBALANCE_FLAGS: RebalanceFlags = RebalanceFlags::new();
static REBALANCE_QUEUES: RebalanceQueues = RebalanceQueues::new();
static REBALANCE_LOCK: Mutex<()> = Mutex::new(());

type Priority = usize;
pub type Link = Option<Rc<RefCell<Thread>>>;


#[repr(align(64))]
struct RebalanceFlag {
    rebalance: bool,
}

impl RebalanceFlag {
    const fn new() -> RebalanceFlag {
        RebalanceFlag { rebalance: false }
    }
}

struct RebalanceFlags {
    flags: [RebalanceFlag; MAX_CPUS],
}

impl RebalanceFlags {
    const fn new() -> RebalanceFlags {
        RebalanceFlags {
            flags : [RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), 
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(),
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), 
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(),
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), 
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(),
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), 
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(),
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), 
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(),
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), 
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(),
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), 
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(),
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), 
                     RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new(), RebalanceFlag::new()],
        }
    }
}

struct RebalanceNode {
    next: usize
}

impl RebalanceNode {
    const fn new() -> RebalanceNode {
        RebalanceNode { next: 0 }
    }
}

struct RebalanceQueues {
    queues: [RebalanceNode; MAX_CPUS],
}

impl RebalanceQueues {
    const fn new() -> RebalanceQueues {
        RebalanceQueues {
            queues: [RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), 
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(),
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), 
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(),
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), 
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(),
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), 
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(),
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), 
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(),
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), 
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(),
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), 
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(),
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), 
                     RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new(), RebalanceNode::new()],
        }
    }
}

pub fn rebalance_thread(mut t: Rc<RefCell<Thread>>) {

   // REBALANCE_QUEUES[
}

enum ThreadState {
    Running = 0,
    Runnable = 1,
    Paused = 2, 
}

const STACK_SIZE_IN_LINES: usize = 4096 * 2;

struct Stack {
    mem: [usize; STACK_SIZE_IN_LINES],
}

pub struct Context {
  r15: usize,
  r14: usize,
  r13: usize, 
  r12: usize,
  r11: usize, 
  rbx: usize, 
  rbp: usize,  
  rsp: usize,
  rflags: usize,
}

pub struct Thread {
    name: String,
    state: ThreadState, 
    priority: Priority, 
    context: Context,
    stack: RefCell<Box<Stack>>,
    // Next thread in the scheduling queue
    next: Link,
    // Next thread on the domain list 
    pub next_domain: Option<Arc<Mutex<Rc<RefCell<Thread>>>>>,
}


struct SchedulerQueue {
    highest: Priority,
    prio_queues: [Link; MAX_PRIO + 1],
}

pub struct Scheduler {
    active: bool,
    active_queue: SchedulerQueue,
    passive_queue: SchedulerQueue,
}

impl Stack {

    pub fn new() -> Stack {
        Stack{mem: [0; STACK_SIZE_IN_LINES]}
    }

}

impl Context {

    pub fn new() -> Context {
        Context{ r15: 0, r14: 0, r13:0, r12:0, r11:0, rbx:0, rbp:0, rsp:0, rflags:0 }
    }
}

impl  Thread {
  
    fn init_stack(&mut self, func: extern fn()) {
       
        /* die() takes one argument lets pass it via r15 and prey */
        self.context.r15 = func as usize;

        /* push die() on the stack where the switch will pick 
         * it up with the ret instruction */
        let mut s = &mut **self.stack.borrow_mut(); 
        s.mem[s.mem.len() - 1] = die as usize;

        /* set the stack pointer to point to die() */
        //self.context.rsp = s.mem[s.mem.len() - 1].as_ptr(); 
        self.context.rsp = &(s.mem[s.mem.len() - 1]) as *const usize as usize;
    }

    pub fn new(name: &str, func: extern fn()) -> Thread  {
        let mut t = Thread {
            name: name.to_string(),
            state: ThreadState::Runnable, 
            priority: 0,
            context: Context::new(),
            stack: RefCell::new(Box::new(Stack::new())),
            next: None, 
            next_domain: None, 
        };

        t.init_stack(func);

        return t; 
    }



}

impl  SchedulerQueue {

    pub const fn new() -> SchedulerQueue {
        SchedulerQueue {
            highest: 0,
            prio_queues: [None, None, None, None, None, None, None, None,
                          None, None, None, None, None, None, None, None],
        }
    }

    fn push_thread(&mut self, queue: usize, mut thread: Rc<RefCell<Thread>>) {
        let previous_head = self.prio_queues[queue].take();

        if let Some(node) = previous_head {
            thread.borrow_mut().next = Some(node);
        }
        self.prio_queues[queue] = Some(thread);
    }

    pub fn pop_thread(&mut self, queue: usize) -> Option<Rc<RefCell<Thread>>> {
        let previous_head = self.prio_queues[queue].take();

        if let Some(mut node) = previous_head {
            self.prio_queues[queue] = node.borrow_mut().next.take();
            Some(node)
        } else {
            None
        }
    }

    // Add thread to the queue that matches thread's priority
    pub fn put_thread(&mut self, mut thread: Rc<RefCell<Thread>>) {
        let prio = thread.borrow_mut().priority;
   
        self.push_thread(prio, thread); 

        if self.highest < prio {
            println!("set highest priority to {}", prio);
            self.highest = prio
        }
    }

    
    // Try to get the thread with the highest priority
    pub fn get_highest(&mut self) -> Option<Rc<RefCell<Thread>>> {
        loop {
            match self.pop_thread(self.highest) {
                None => {
                    if self.highest == 0 {
                        return None;
                    }
                    self.highest += 1;
                },
                Some(t) => {
                    return Some(t);
                },
            }
        }
    }

}

impl  Scheduler {

    pub const fn new() -> Scheduler {
        Scheduler {
            active: true,
            active_queue: SchedulerQueue::new(),
            passive_queue: SchedulerQueue::new(),
        }
    }

    pub fn put_thread(&mut self, mut thread: Rc<RefCell<Thread>>) {
        /* put thread in the currently passive queue */
        if !self.active {
            self.active_queue.put_thread(thread)
        } else {
            self.passive_queue.put_thread(thread)
        }
    }

    fn get_next_active(&mut self) -> Option<Rc<RefCell<Thread>>> {
        if self.active {
            //println!("get highest from active");
            self.active_queue.get_highest()
        } else {
            //println!("get highest from passive");
            self.passive_queue.get_highest()
        }
    }

    
    pub fn get_next(&mut self) -> Option<Rc<RefCell<Thread>>> {
        return self.get_next_active();
    }   

    // Flip active and passive queue making active queue passive
    pub fn flip_queues(&mut self) {
        //println!("flip queues");
        if self.active {
            self.active = false
        } else {
            self.active = true
        }
    }
    
    pub fn next(&mut self) -> Option<Rc<RefCell<Thread>>> {
        if let Some(t) = self.get_next() {
            return Some(t);
        }
        
        // No luck finding a thread in the active queue
        // flip active and passive queues and try again
        self.flip_queues();
        
        if let Some(t) = self.get_next() {
            return Some(t);
        }
       
        return None;
    }
}


/// Just make sure die follows C calling convention
/// We don't really need it now as we pass the function pointer via r15
#[no_mangle] 
extern "C" fn die(/*func: extern fn()*/) {
    let func: extern fn();

    /* For now prey its still in r15 */
    unsafe{
        asm!("mov $0, r15" : "=r"(func) : : "memory" : "intel", "volatile");
    };

    println!("Starting new thread"); 

    // Enable interrupts so we get next scheduling tick
    x86_64::instructions::interrupts::enable();
    func();
    
    loop {
        println!("waiting to be cleaned up"); 
    };
}


/// Switch to the next context by restoring its stack and registers
#[cold]
#[inline(never)]
#[naked]
pub unsafe fn switch(prev: *mut Thread, next: *mut Thread) {
    //asm!("fxsave64 [$0]" : : "r"(self.fx) : "memory" : "intel", "volatile");
    //self.loadable = true;
    //if next.loadable {
    //    asm!("fxrstor64 [$0]" : : "r"(next.fx) : "memory" : "intel", "volatile");
    //}else{
    //    asm!("fninit" : : : "memory" : "intel", "volatile");
    //}

    //asm!("mov $0, cr3" : "=r"(self.cr3) : : "memory" : "intel", "volatile");
    //if next.cr3 != self.cr3 {
    //    asm!("mov cr3, $0" : : "r"(next.cr3) : "memory" : "intel", "volatile");
    //}

    asm!("pushfq ; pop $0" : "=r"((*prev).context.rflags) : : "memory" : "intel", "volatile");
    asm!("push $0 ; popfq" : : "r"((*next).context.rflags) : "memory" : "intel", "volatile");

    asm!("mov $0, rbx" : "=r"((*prev).context.rbx) : : "memory" : "intel", "volatile");
    asm!("mov rbx, $0" : : "r"((*next).context.rbx) : "memory" : "intel", "volatile");

    asm!("mov $0, r12" : "=r"((*prev).context.r12) : : "memory" : "intel", "volatile");
    asm!("mov r12, $0" : : "r"((*next).context.r12) : "memory" : "intel", "volatile");

    asm!("mov $0, r13" : "=r"((*prev).context.r13) : : "memory" : "intel", "volatile");
    asm!("mov r13, $0" : : "r"((*next).context.r13) : "memory" : "intel", "volatile");

    asm!("mov $0, r14" : "=r"((*prev).context.r14) : : "memory" : "intel", "volatile");
    asm!("mov r14, $0" : : "r"((*next).context.r14) : "memory" : "intel", "volatile");

    asm!("mov $0, r15" : "=r"((*prev).context.r15) : : "memory" : "intel", "volatile");
    asm!("mov r15, $0" : : "r"((*next).context.r15) : "memory" : "intel", "volatile");

    asm!("mov $0, rsp" : "=r"((*prev).context.rsp) : : "memory" : "intel", "volatile");
    asm!("mov rsp, $0" : : "r"((*next).context.rsp) : "memory" : "intel", "volatile");

    asm!("mov $0, rbp" : "=r"((*prev).context.rbp) : : "memory" : "intel", "volatile");
    asm!("mov rbp, $0" : : "r"((*next).context.rbp) : "memory" : "intel", "volatile");
}

fn set_current(mut t: Rc<RefCell<Thread>>) {
    CURRENT.replace(Some(t)); 
}

//fn get_current_ref() -> &'static mut Option<Box<Thread>> {
//    unsafe{&mut *CURRENT.get()}
//}

fn get_current() -> Option<Rc<RefCell<Thread>>> {
    CURRENT.replace(None)
}


// Kicked from the timer IRQ
pub fn schedule() {

    //println!("Schedule"); 

    let mut s = SCHED.borrow_mut();
    let mut next_thread = match s.next() {
        Some(t) => t,
        None => {
            // Nothing again, current is the only runnable thread, no need to
            // context switch
            println!("No runnable threads");
            return; 
        }

    };

    let mut c = match get_current() {
        Some(t) => t,
        None => { return; } 
    };


    // Rc<RefCell<Thread>
   // let prev = &mut *c.as_ptr() as *mut Thread; 
    //let next = &mut *next_thread.as_ptr() as *mut Thread; 

    let prev = c.as_ptr(); 
    let next = next_thread.as_ptr(); 


    // Make next thread current
    set_current(next_thread); 

    // put the old thread back in the scheduling queue
    s.put_thread(c);

    drop(s); 

    unsafe {
        switch(prev, next);
    }

}


// yield is a reserved keyword
pub fn do_yield() {
    println!("Yield"); 
    schedule();
}

pub extern fn idle() {
    halt(); 
}

pub fn create_thread (name: &str, func: extern fn()) -> Box<dyn syscalls::Thread> {
    let mut s = SCHED.borrow_mut();

    let mut t = Rc::new(RefCell::new(Thread::new(name, func)));
 
    let mut pt = Box::new(PThread::new(Rc::clone(&t)));
   
    s.put_thread(t);
    return pt; 
}

struct PThread {
    t: Rc<RefCell<Thread>>
}

impl PThread {
    pub const fn new(t:Rc<RefCell<Thread>>) -> PThread {
        PThread {
            t: t,
        }
    }
}

impl syscalls::Thread for PThread {
    fn set_affinity(&self, affinity: u64) {
        disable_irq();
        println!("Setting affinity:{} for {}", affinity, self.t.borrow().name); 
        enable_irq(); 
    }
}

pub fn init_threads() {
    let mut idle = Rc::new(RefCell::new(Thread::new("idle", idle)));

    // Make idle the current thread
    set_current(idle);   
}

