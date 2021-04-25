#![no_main]
#![no_std]
#![feature(asm)]
#![feature(naked_functions)]
// openocd -f interface/stlink-v2-1.cfg -f target/stm32f4x.cfg
use panic_halt as _;
use stm32f407g_disc as board;
use crate::board::{
    hal::stm32,
    hal::{prelude::*},
    led::{Leds},
};
use core::cell::RefCell;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicUsize, Ordering};
use cortex_m::{
    peripheral::Peripherals, 
    interrupt as Interrupt, 
};
use cortex_m::interrupt::{Mutex};
use cortex_m_rt::{entry, exception};
use cortex_m::{self, peripheral::syst::SystClkSource};

const CTX_STACK_START: u32 = 0x2001_D000; // 0x2001_C000;
const CTX_STACK_SIZE: u32 = 40;             // 10 regs to store (r4-r11, lr)
const TASK_STACK_START: u32 = 0x2001_CFFF; // 0x2001_BFFF;
// const TASK_STACK_END: u32 = 0x2000_0000;
const TASK_STACK_SIZE: u32 = 2048;
const TASKS_LIMIT: u32 = 2; // (TASK_STACK_START-TASK_STACK_END)/TASK_STACK_SIZE;   // 58 in this case


static LEDS: Mutex<RefCell<Option<Leds>>> = Mutex::new(RefCell::new(None));
static NTASKS: AtomicUsize = AtomicUsize::new(0);
static CURRTASK: AtomicUsize = AtomicUsize::new(0);
static GLOBALTIME: AtomicUsize = AtomicUsize::new(0);


#[exception]
fn SysTick() {
    schedule();
    let mut time = GLOBALTIME.load(Ordering:: Relaxed);
    time = if time == 200 {0} else {time+1};
    GLOBALTIME.store(time, Ordering::Relaxed);
}

extern "C" fn schedule() {
    let ntasks: u32 = NTASKS.load(Ordering::Relaxed) as u32;
    let currtask: u32 = CURRTASK.load(Ordering::Relaxed) as u32;
    let nexttask: u32 = (currtask + 1)%TASKS_LIMIT as u32;
    CURRTASK.store(nexttask as usize, Ordering::Relaxed);
    
    // ideally want to check if it exceeds TASKS_LIMIT before adding a new task
    if ntasks < 2 { 
        let to: u32 = CTX_STACK_START + (currtask + 1)*CTX_STACK_SIZE; // curr
        let func_ptr: u32 = if currtask == 0 { FlashBlue as u32 } else { FlashRed as u32 };
        let sp_addr: u32 = TASK_STACK_START - nexttask*TASK_STACK_SIZE;
        NTASKS.store((ntasks + 1) as usize, Ordering::Relaxed);

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
            
            // "stmdb r1!, {{r4-r11, lr}}",
            // "ldmia r0!, {{r4-r11, lr}}",
            // "ldr r3, [r0], #4",
            // "msr msp, r3",
            // "bx lr",  // bx or blx
            options(noreturn)
        );
    }
}

#[allow(non_snake_case)]
extern "C" fn FlashBlue() {
    let mut count = 0;
    loop {
        Interrupt::free(|cs| {
            if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
                if count < 100 {
                    leds[3].on();
                }
                else if count >= 100 {
                    leds[3].off();
                }
            }
        });
        let time = GLOBALTIME.load(Ordering::Relaxed);
        count = time;
    }
}

#[allow(non_snake_case)]
extern "C" fn FlashRed() {
    let mut count = 0;
    loop {
        Interrupt::free(|cs| {
            if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
                if count < 100 {
                    leds[2].on();
                }
                else if count >= 100 {
                    leds[2].off();
                }
            }
        });
        let time = GLOBALTIME.load(Ordering::Relaxed);
        count = time;
    }
}

#[entry]
fn main() -> ! {
    if let (Some(p), Some(mut cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        
        let syst = &mut cp.SYST;
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(1_680_000); // 0.1s interrupts
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();
        
        p.RCC.apb2enr.write(|w| w.syscfgen().enabled());
        // Configure clock to 168 MHz (i.e. the maximum) and freeze it
        let rcc = p.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(168.mhz()).freeze();

        // Initialize on-board LEDs
        let gpiod = p.GPIOD.split();
        let leds = Leds::new(gpiod);

        // Put peripherals into gloabl statics
        Interrupt::free(|cs| {
            LEDS.borrow(cs).replace(Some(leds));
        });

        loop { continue; }
    }
    loop { continue; }
}
