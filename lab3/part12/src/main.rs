#![no_main]
#![no_std]

use panic_halt as _;
use stm32f407g_disc as board;

use crate::board::{
    hal::stm32,
    hal::{prelude::*, interrupt, timer::{Timer, Event}},
    led::{Leds},
    stm32::NVIC,
    gpio::{Edge, Floating, Input, ExtiPin, gpioa::PA0, gpioc::PC7}, 
    EXTI, GPIOC,
};

use core::cell::RefCell;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicUsize, Ordering};
use cortex_m::{peripheral::Peripherals, interrupt as Interrupt};
use cortex_m::interrupt::{Mutex};
use cortex_m_rt::entry;


static PAUSED: AtomicUsize = AtomicUsize::new(0);
static LIGHT: AtomicUsize = AtomicUsize::new(3);
static LEDS: Mutex<RefCell<Option<Leds>>> = Mutex::new(RefCell::new(None));
static TIM2: Mutex<RefCell<Option<Timer<stm32f407g_disc::stm32::TIM2>>>> = Mutex::new(RefCell::new(None));
static BUTTON: Mutex<RefCell<Option<PA0<Input<Floating>>>>> = Mutex::new(RefCell::new(None));
static EXTI: Mutex<RefCell<Option<EXTI>>> = Mutex::new(RefCell::new(None));
static TOUCHPIN: Mutex<RefCell<Option<PC7<Input<Floating>>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn TIM2() {
    Interrupt::free(|cs| {
        if let (Some(ref mut tim2), Some(ref mut leds)) = (
            TIM2.borrow(cs).borrow_mut().deref_mut(),
            LEDS.borrow(cs).borrow_mut().deref_mut(),
        ) {
            tim2.clear_interrupt(Event::TimeOut);
            
            let paused: usize = PAUSED.load(Ordering::Relaxed);
            if paused == 0 {
                let tmp: usize = LIGHT.load(Ordering::Relaxed);
                LIGHT.store((tmp+1)%4, Ordering::Relaxed);
                leds[tmp].off();
                leds[(tmp+1)%4].on();
            }
        }
    });
}

#[interrupt]
fn EXTI0() {
    Interrupt::free(|cs| {
        if let (Some(ref mut button), Some(ref mut exti), Some(ref mut leds)) = (
            BUTTON.borrow(cs).borrow_mut().deref_mut(),
            EXTI.borrow(cs).borrow_mut().deref_mut(),
            LEDS.borrow(cs).borrow_mut().deref_mut(),
        ) {
            button.clear_interrupt_pending_bit(exti);
            
            let paused: usize = PAUSED.load(Ordering::Relaxed);
            if button.is_high().unwrap() {
                PAUSED.store(1, Ordering::Relaxed);
            }
            else {
                let tmp: usize = LIGHT.load(Ordering::Relaxed);
                LIGHT.store(0, Ordering::Relaxed);
                leds[tmp].off();
                leds[0].on();
                PAUSED.store(0, Ordering::Relaxed);
            }
        }
    });
}

#[interrupt]
fn EXTI9_5() {
    Interrupt::free(|cs| {
        if let (Some(ref mut touchpin), Some(ref mut exti), Some(ref mut leds)) = (
            TOUCHPIN.borrow(cs).borrow_mut().deref_mut(),
            EXTI.borrow(cs).borrow_mut().deref_mut(),
            LEDS.borrow(cs).borrow_mut().deref_mut(),
        ) {
            touchpin.clear_interrupt_pending_bit(exti);
            
            let paused: usize = PAUSED.load(Ordering::Relaxed);
            if paused == 0 {
                PAUSED.store(1, Ordering::Relaxed);
            }
            else {
                PAUSED.store(0, Ordering::Relaxed);
            }
        }
    });
}

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        
        // Set NVIC interrupts
        let mut nvic = cp.NVIC;
        unsafe { 
            NVIC::unmask(interrupt::TIM2);
            NVIC::unmask(interrupt::EXTI0);
            NVIC::unmask(interrupt::EXTI9_5);
            nvic.set_priority(interrupt::TIM2, 1u8);
            nvic.set_priority(interrupt::EXTI0, 2_u8);
            nvic.set_priority(interrupt::EXTI9_5, 3_u8);
        } 

        // Configure clock to 168 MHz (i.e. the maximum) and freeze it
        p.RCC.apb2enr.write(|w| w.syscfgen().enabled()); // enable syscfg timer
        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(168.mhz()).freeze();

        // Initialize on-board LEDs
        let gpiod = p.GPIOD.split();
        let mut leds = Leds::new(gpiod);

        // Initialize tim2 to 0.5s timeouts
        let mut mytim2 = Timer::tim2(p.TIM2, 2.hz(), clocks);
        mytim2.listen(Event::TimeOut);

        // Initialize PA0  User button
        let gpioa = p.GPIOA.split();
        let mut button = gpioa.pa0.into_floating_input();
        button.make_interrupt_source(&mut p.SYSCFG);
        button.enable_interrupt(&mut p.EXTI);
        button.trigger_on_edge(&mut p.EXTI, Edge::RISING_FALLING);
        
        // Initialize PC7 Touch Pin
        let gpioc = p.GPIOC.split();
        let pc7 = gpioc.pc7;
        let mut touchpin = pc7.into_push_pull_output();
        touchpin.set_high();
        let mut touchpin = touchpin.into_floating_input();
        
        touchpin.make_interrupt_source(&mut p.SYSCFG);
        touchpin.enable_interrupt(&mut p.EXTI);
        touchpin.trigger_on_edge(&mut p.EXTI, Edge::FALLING);
        
        // Put peripherals into gloabl statics
        let exti = p.EXTI;
        Interrupt::free(|cs| {
            LEDS.borrow(cs).replace(Some(leds));
            TIM2.borrow(cs).replace(Some(mytim2));
            BUTTON.borrow(cs).replace(Some(button));
            EXTI.borrow(cs).replace(Some(exti));
            TOUCHPIN.borrow(cs).replace(Some(touchpin))
        });

        loop { continue; }
    }
    loop { continue; }
}
