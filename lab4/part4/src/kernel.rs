

// const NTASKS: usize = 2;
use cortex_m_rt::{exception};
use cortex_m::{self, peripheral::syst::SystClkSource};
use cortex_m::{
    peripheral::Peripherals, 
    interrupt as Interrupt, 
};
use cortex_m::interrupt::{Mutex};
use core::cell::RefCell;
use core::ops::DerefMut;

use core::sync::atomic::{AtomicUsize, Ordering};

const CTX_STACK_START: u32 = 0x2001_D000; 
const CTX_STACK_SIZE: u32 = 40;             // 10 regs to store (r4-r11, lr)
const TASK_STACK_START: u32 = 0x2001_CFFF; 
// const TASK_STACK_END: u32 = 0x2000_0000;
const TASK_STACK_SIZE: u32 = 2048;
const TASKS_LIMIT: u32 = 32; // (TASK_STACK_START-TASK_STACK_END)/TASK_STACK_SIZE;   // 58 in this case

static CURRTASK: AtomicUsize = AtomicUsize::new(0);
static GLOBALTIME: AtomicUsize = AtomicUsize::new(0);
static STARTED: AtomicUsize = AtomicUsize::new(0);
static NTASKS: AtomicUsize = AtomicUsize::new(0);

static FUNC_PTRS: Mutex<RefCell<Option<[u32; TASKS_LIMIT as usize]>>> = Mutex::new(RefCell::new(None));


pub fn init() {
    Interrupt::free(|cs| {
        let arr: [u32; TASKS_LIMIT as usize] = [0; TASKS_LIMIT as usize];
        FUNC_PTRS.borrow(cs).replace(Some(arr));
    });
}

pub fn start() {
    if let Some(mut cp) = Peripherals::take() {
        let syst = &mut cp.SYST;
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(1_680_000); // 0.1s interrupts
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();
    }
}

pub fn delay(time: usize) {
    let start: usize = GLOBALTIME.load(Ordering::Relaxed);
    let target: usize = time + start;

    if target > usize::MAX {
        delay(usize::MAX - start);
        delay(time - usize::MAX + start);
        return;
    }

    while GLOBALTIME.load(Ordering::Relaxed) < target {
        continue;
    }
}

pub fn create_task(func: u32) {
    let ntasks: usize = NTASKS.load(Ordering::Relaxed);

    if ntasks + 1 >= TASKS_LIMIT as usize {
        return;
    }

    NTASKS.store(ntasks + 1, Ordering::Relaxed);

    Interrupt::free(|cs| {
        if let Some(ref mut func_ptrs) = FUNC_PTRS.borrow(cs).borrow_mut().deref_mut() {
            func_ptrs[ntasks] = func;
        }
    });
}

#[exception]
fn SysTick() {
    schedule();
    GLOBALTIME.fetch_add(1, Ordering:: Relaxed);
}

extern "C" fn schedule() {
    let ntasks: usize = NTASKS.load(Ordering::Relaxed);
    let currtask: u32 = CURRTASK.load(Ordering::Relaxed) as u32;
    let nexttask: u32 = (currtask + 1)%(ntasks as u32);
    CURRTASK.store(nexttask as usize, Ordering::Relaxed);
    let started: usize = STARTED.load(Ordering::Relaxed);

    if started < ntasks { 
        let to: u32 = CTX_STACK_START + (currtask + 1)*CTX_STACK_SIZE; // curr
        
        let mut func_ptr: u32 = 0;
        Interrupt::free(|cs| {
            if let Some(ref mut func_ptrs) = FUNC_PTRS.borrow(cs).borrow_mut().deref_mut() {
                func_ptr = func_ptrs[started];
            }
        });

        let sp_addr: u32 = TASK_STACK_START - nexttask*TASK_STACK_SIZE;
        STARTED.store((started + 1) as usize, Ordering::Relaxed);

        add_task(to, func_ptr, sp_addr);
    }
    else {
        let from: u32 = CTX_STACK_START + (nexttask)*CTX_STACK_SIZE;  
        let to: u32 = CTX_STACK_START + (currtask+1)*CTX_STACK_SIZE;
        context_switch(from, to);
    }
}

#[naked]
extern "C" fn add_task(_to: u32, _func_ptr: u32, _sp_addr: u32) {
    unsafe {
        asm!(
            "mrs r3, msp",
            "str r3, [r0, #-4]!",
            "stmdb r0!, {{r4-r11, lr}}",

            "msr msp, r2",
            
            "mov r3, #16777216",             // xPSR 1 << 24
            "str r3, [sp, #-4]!",            // xPSR
            
            "str r1, [sp, #-4]!",            // ret address (to new func) 

            "mov lr, {exc_return}",         // lr 0xFFFFFFF9
            "str lr, [sp, #-4]!",           // lr
            
            "mov r3, #0",
            "str r3, [sp, #-4]!",
            "str r3, [sp, #-4]!",
            "str r3, [sp, #-4]!",
            "str r3, [sp, #-4]!",
            "str r3, [sp, #-4]!",
            
            "bx lr",  // bx or blx
            exc_return = const (0xFFFFFFF9 as u32),
            options(noreturn)
        );
    }
}

#[naked]
extern "C" fn context_switch(_from: u32, _to: u32) {
    // 1. save current stack pointer
    // 2. save current r4-r11, lr
    // 3. load new stack pointer
    // 4. load new r4-r11, lr
    // 5. return to *lr

    unsafe {    
        asm!(
            "mrs r3, msp",
            "str r3, [r1, #-4]!",
            "str lr, [r1, #-4]!",
            "str r11, [r1, #-4]!",
            "str r10, [r1, #-4]!",
            "str r9, [r1, #-4]!",
            "str r8, [r1, #-4]!",
            "str r7, [r1, #-4]!",
            "str r6, [r1, #-4]!",
            "str r5, [r1, #-4]!",
            "str r4, [r1, #-4]!",

            "ldr r4, [r0], #4",
            "ldr r5, [r0], #4",
            "ldr r6, [r0], #4",
            "ldr r7, [r0], #4",
            "ldr r8, [r0], #4",
            "ldr r9, [r0], #4",
            "ldr r10, [r0], #4",
            "ldr r11, [r0], #4",
            "ldr lr, [r0], #4",
            "ldr r3, [r0], #4",
            "msr msp, r3",
            "bx lr",
            
            options(noreturn)
        );
    }
}

