#![no_main]
#![no_std]
#![feature(asm)]
#![feature(naked_functions)]
// openocd -f interface/stlink-v2-1.cfg -f target/stm32f4x.cfg
use panic_halt as _;
use stm32f407g_disc as board;
#[allow(unused_imports)]
use crate::board::{
    hal::stm32,
    hal::{prelude::*, interrupt, timer::{Timer, Event}},
    led::{Leds},
    stm32::NVIC,
    gpio::{Edge, Floating, Input, ExtiPin, gpioa::PA0, gpioc::PC7}, 
    EXTI, GPIOC,
};
#[allow(unused_imports)]
use core::cell::RefCell;
use core::ops::DerefMut;
#[allow(unused_imports)]
use core::sync::atomic::{AtomicUsize, Ordering};
use cortex_m::{
    peripheral::Peripherals, 
    interrupt as Interrupt, 
    asm::wfi
    // peripheral::SCB::{set_pendsv, clear_pendsv}
};
use cortex_m::interrupt::{Mutex};
use cortex_m_rt::{entry, exception};


static LEDS: Mutex<RefCell<Option<Leds>>> = Mutex::new(RefCell::new(None));
static TIM2: Mutex<RefCell<Option<Timer<stm32f407g_disc::stm32::TIM2>>>> = Mutex::new(RefCell::new(None));
static EXTI: Mutex<RefCell<Option<EXTI>>> = Mutex::new(RefCell::new(None));

static NTASKS: AtomicUsize = AtomicUsize::new(1);
// static CURRTASK: AtomicUsize = AtomicUsize::new(0);

// static TCBPTRS: Mutex<RefCell<Option<[u32;2]>>> = Mutex::new(RefCell::new(None));

#[exception]
fn PendSV () {
    // unsafe {
    //     asm!(
    //         "mrs r0, msp",
    //         "isb",
    //         "sub r0, r0, #1024",
    //         "mov r9, {x}",
    //         "mov r6, {reset}",
    //         "mov r5, {reset}",
    //         "mov r4, {reset}",
    //         "mov r3, {reset}",
    //         "mov r2, {reset}",
    //         "mov r1, {reset}",
    //         "stmdb r0!, {{r1-r6,r8-r9}}",
    //         "msr msp, r0",
    //         "isb",

    //         x = const 16777216, // 1 << 24
    //         reset = const 0,
    //         // exc_return = const 0xFFFFFFF9 as u32,
    //         in("r8") FlashRed as u32
    //     );
    // }
    // cortex_m::peripheral::SCB::clear_pendsv();
}

#[interrupt]
fn TIM2() {
    // let ntasks: usize = NTASKS.load(Ordering::Relaxed);
    // Interrupt::free(|cs| {
    //     if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
    //         // let ntasks: usize = NTASKS.load(Ordering::Relaxed);
    //         if ntasks == 1 {
    //             leds[0].on();
    //         }
    //         leds[0].on();
                
    //     }
    // });

    // if ntasks == 1 {
    //     NTASKS.store(2, Ordering::Relaxed);

    //     unsafe {
    //         asm!(
    //             "mrs r0, msp",
    //             "isb",
    //             // "sub r0, r0, #4",
    //             "mov r10, {x}",
    //             // "mov r9, #FlashRed",
    //             "mov r8, #8",
    //             "mov r5, #5",
    //             "mov r4, #4",
    //             "mov r3, #3",
    //             "mov r2, #2",
    //             "mov r1, #1",
    //             "stmdb r0!, {{r1-r5,r8-r10}}",
    //             "msr msp, r0",
    //             "isb",

    //             x = const 16777216, // 1 << 24
    //             // reset = const 0,
    //             // exc_return = const 0xFFFFFFF9 as u32,
    //             in("r9") FlashRed as u32,
    //             // out("r1") _,
    //             // out("r2") _,
    //             // out("r3") _,
    //             // out("r4") _,
    //             // out("r5") _,
    //             // out("r6") _,
    //             // out("r9") _,
    //         );
    //     }
    // }

    Interrupt::free(|cs| {
        if let Some(ref mut tim2) = TIM2.borrow(cs).borrow_mut().deref_mut() {
            tim2.clear_interrupt(Event::TimeOut);
        }
    });
    // cortex_m::peripheral::SCB::set_pendsv();
    
}

#[allow(non_snake_case)]
extern "C" fn FlashBlue() {
    // let mut count = 0;
    
    // loop {
    //     Interrupt::free(|cs| {
    //         if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
    //             if count == 0 {
    //                 leds[3].on();
    //             }
    //             else if count == 100 {
    //                 leds[3].off();
    //             }
    //             else if count == 200 {
    //                 count = -1;
    //             }
    //             count += 1;
    //         }
    //     });
        wfi();
    // }
}

#[allow(non_snake_case)]
extern "C" fn FlashRed() {
    let mut count = 0;
    loop {
        Interrupt::free(|cs| {
            if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
                if count == 0 {
                    leds[2].on();
                }
                else if count == 100 {
                    leds[2].off();
                }
                else if count == 200 {
                    count = -1;
                }
                count += 1;
            }
        });
        wfi();
    }
}

#[allow(unused_mut)]
#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        // Set NVIC interrupts
        let mut nvic = cp.NVIC;
        unsafe { 
            NVIC::unmask(interrupt::TIM2);
            nvic.set_priority(interrupt::TIM2, 1u8);
        } 
        p.RCC.apb2enr.write(|w| w.syscfgen().enabled());
        // Configure clock to 168 MHz (i.e. the maximum) and freeze it
        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(168.mhz()).freeze();

        // Initialize on-board LEDs
        let gpiod = p.GPIOD.split();
        let mut leds = Leds::new(gpiod);

        // Initialize tim2 to 1s timeouts
        let mut mytim2 = Timer::tim2(p.TIM2, 100.hz(), clocks);
        mytim2.listen(Event::TimeOut);


        // let offset:u32 = 0x00000400;    // 1024
        // let spstart:u32 = 0x2001FFFF;
        // let mut tcbptrs: [u32;2] = [spstart; 2];
        // // let mut stack1 = [0x00000000; 1024];
        // // let mut stack2 = [0x00000000; 1024]; 
        // // tcbptrs[0] = create_tcb(&mut stack0, FlashBlue);
        // // tcbptrs[1] = create_tcb(&mut stack1, FlashRed);
        // for i in 0..2 {
        //     tcbptrs[i] = spstart - ((i + 1) as u32 *offset);
        // }

        // Put peripherals into gloabl statics
        let exti = p.EXTI;
        Interrupt::free(|cs| {
            LEDS.borrow(cs).replace(Some(leds));
            TIM2.borrow(cs).replace(Some(mytim2));
            EXTI.borrow(cs).replace(Some(exti));
            // TCBPTRS.borrow(cs).replace(Some(tcbptrs));
        });

        FlashBlue();
        // FlashRed();

        loop { continue; }
    }
    loop { continue; }
}

// fn create_tcb(
//     stack: &mut [u32],
//     handler: fn() -> !,
// ) -> u32 {
//     if stack.len() < 32 {
//         return Err(ERR_STACK_TOO_SMALL);
//     }
//     let idx = stack.len() - 1;
//     stack[idx] = 1 << 24; // xPSR
//     let pc: usize = unsafe { core::intrinsics::transmute(handler as *const fn()) };
//     stack[idx - 1] = pc as u32; // PC
//     stack[idx - 2] = 0xFFFFFFF9; // LR
//     stack[idx - 3] = 0xCCCCCCCC; // R12
//     stack[idx - 4] = 0x33333333; // R3
//     stack[idx - 5] = 0x22222222; // R2
//     stack[idx - 6] = 0x11111111; // R1
//     stack[idx - 7] = 0x00000000; // R0
//     stack[idx - 8] = 0x77777777; // R7
//     stack[idx - 9] = 0x66666666; // R6
//     stack[idx - 10] = 0x55555555; // R5
//     stack[idx - 11] = 0x44444444; // R4
//     stack[idx - 12] = 0xBBBBBBBB; // R11
//     stack[idx - 13] = 0xAAAAAAAA; // R10
//     stack[idx - 14] = 0x99999999; // R9
//     stack[idx - 15] = 0x88888888; // R8
//     // unsafe {
//     let sp: usize = core::intrinsics::transmute(&stack[stack.len() - 16]);
//     return sp as u32;
//     // }