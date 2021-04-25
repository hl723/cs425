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
use cortex_m::{
    interrupt as Interrupt, 
};
use cortex_m::interrupt::{Mutex};
use cortex_m_rt::{entry};

mod kernel;

static LEDS: Mutex<RefCell<Option<Leds>>> = Mutex::new(RefCell::new(None));

#[allow(non_snake_case)]
extern "C" fn FlashBlue() {

    loop {
        Interrupt::free(|cs| {
            if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
                leds[3].toggle();
            }
        });
        kernel::delay(100);
    }
}

#[allow(non_snake_case)]
extern "C" fn FlashRed() {
    loop {
        Interrupt::free(|cs| {
            if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
                leds[2].toggle();
            }
        });
        kernel::delay(100);
    }
}

#[allow(non_snake_case)]
extern "C" fn FlashOrange() {
    loop {
        Interrupt::free(|cs| {
            if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
                leds[1].toggle();
            }
        });
        kernel::delay(100);
    }
}

#[allow(non_snake_case)]
extern "C" fn FlashGreen() {
    loop {
        Interrupt::free(|cs| {
            if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
                leds[0].toggle();
            }
        });
        kernel::delay(100);
    }
}

#[entry]
fn main() -> ! {
    if let Some(p) = stm32::Peripherals::take() {
        
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
    }

    kernel::init();
    kernel::create_task(FlashBlue as u32);
    kernel::create_task(FlashRed as u32);
    // kernel::create_task(FlashOrange as u32);
    // kernel::create_task(FlashGreen as u32);
    kernel::start();

    loop { continue; }
}
