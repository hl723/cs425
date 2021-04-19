#![no_main]
#![no_std]
#![feature(asm)]

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
use cortex_m::{peripheral::Peripherals, interrupt as Interrupt, asm::wfi};
use cortex_m::interrupt::{Mutex};
use cortex_m_rt::entry;


static LEDS: Mutex<RefCell<Option<Leds>>> = Mutex::new(RefCell::new(None));
static TIM2: Mutex<RefCell<Option<Timer<stm32f407g_disc::stm32::TIM2>>>> = Mutex::new(RefCell::new(None));
static EXTI: Mutex<RefCell<Option<EXTI>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn TIM2() {
    Interrupt::free(|cs| {
        if let Some(ref mut tim2) = TIM2.borrow(cs).borrow_mut().deref_mut() {
            tim2.clear_interrupt(Event::TimeOut);
        }
    });
}

fn FlashBlue() {
    let mut count = 0;
    while 0 == 0 {
        Interrupt::free(|cs| {
            if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
                wfi();
                count += 1;
                if count == 1000 {
                    count = 0;
                    let mut state:u32;
                    unsafe {
                        asm!(
                            "sub sp, sp, #1000", // no clue why #1000 works but #256 does not
                            "ldr {0}, [sp], #4",
                            "mov {1}, #1",
                            "cmp {0}, #1",
                            "it eq",
                            "moveq {1}, #0",
                            "str {1}, [sp, #-4]!",
                            "add sp, sp, #1000",
                            out(reg) state,
                            out(reg) _,
                        );
                    }
                    if state == 0 {
                        leds[3].on();
                    }
                    else {
                        leds[3].off();
                    }
                }
            }
        });
    }
}

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        
        // Set NVIC interrupts
        let mut nvic = cp.NVIC;
        unsafe { 
            NVIC::unmask(interrupt::TIM2);
            nvic.set_priority(interrupt::TIM2, 1u8);
        } 

        // Configure clock to 168 MHz (i.e. the maximum) and freeze it
        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(168.mhz()).freeze();

        // Initialize on-board LEDs
        let gpiod = p.GPIOD.split();
        let mut leds = Leds::new(gpiod);

        // Initialize tim2 to 1s timeouts
        let mut mytim2 = Timer::tim2(p.TIM2, 1000.hz(), clocks);
        mytim2.listen(Event::TimeOut);
        
        leds[3].on();
        // Put peripherals into gloabl statics
        let exti = p.EXTI;
        Interrupt::free(|cs| {
            LEDS.borrow(cs).replace(Some(leds));
            TIM2.borrow(cs).replace(Some(mytim2));
            EXTI.borrow(cs).replace(Some(exti));
        });

        unsafe {
            asm!(
                "sub sp, sp, #256",
                "mov {0}, #1",
                "str {0}, [sp, #-4]!",
                "add sp, sp, #256",
                out(reg) _,
            );
        }
    
        FlashBlue();

        loop { continue; }
    }
    loop { continue; }
}
