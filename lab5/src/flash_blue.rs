#![no_main]
#![no_std]
#![feature(asm)]
use panic_halt as _;
use stm32f407g_disc as board;
use crate::board::{
    hal::{prelude::*},
};
use cortex_m_rt::{entry};


#[no_mangle]
#[allow(non_snake_case)]
fn flash_blue() -> ! {
    loop {
        unsafe {
            asm!(
                // toggle blue LED
                "mov r0, #2",
                "mov r1, #3",
                "svc #0",

                // // delay for 100
                // "mov r0, #1",   
                // "mov r1, #100",
                // "svc #0",
            );
        }

        unsafe {
            asm!(
                // // toggle blue LED
                // "mov r0, #2",
                // "mov r1, #3",
                // "svc #0",

                // // delay for 100
                "mov r0, #1",   
                "mov r1, #100",
                "svc #0",
            );
        }
    }
}

#[entry]
fn main() -> ! {
    let a: u32 = flash_blue as u32;
    loop { continue; }
}