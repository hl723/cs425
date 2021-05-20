#![no_main]
#![no_std]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]

// openocd -f interface/stlink-v2-1.cfg -f target/stm32f4x.cfg
use panic_halt as _;
use stm32f407g_disc as board;
use crate::board::{
    hal::stm32,
    hal::{delay::Delay, prelude::*},
    
    led::{Leds},
};
use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m::{
    interrupt as Interrupt, 
    peripheral::Peripherals
};
use cortex_m::interrupt::{Mutex};
use cortex_m_rt::{entry, exception};


use object::{Object, ObjectSymbol, ObjectSegment};
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;

mod kernel;

use crate::kernel::{LEDS, DELAY};

#[macro_use]
mod macros {
    #[repr(C)] // guarantee 'bytes' comes after '_align'
    pub struct AlignedAs<Align, Bytes: ?Sized> {
        pub _align: [Align; 0],
        pub bytes: Bytes,
    }

    macro_rules! include_bytes_align_as {
        ($align_ty:ty, $path:literal) => {
            {  // const block expression to encapsulate the static
                use $crate::macros::AlignedAs;
                
                // this assignment is made possible by CoerceUnsized
                static ALIGNED: &AlignedAs::<$align_ty, [u8]> = &AlignedAs {
                    _align: [],
                    bytes: *include_bytes!($path),
                };
    
                &ALIGNED.bytes
            }
        };
    }
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();


#[exception]
fn SVCall() {
    let mut sysnum: u32;
    let mut arg: usize;
    unsafe {
        asm!(
            "mov r2, #0",
            out("r0") sysnum,
            out("r1") arg,
        );
    }

    match sysnum {
        1 => delay(arg),
        2 => toggle_led(arg),
        _ => {},
    }

    // let mut out_func = 0;
    // match sysnum {
    //     1 => out_func = delay as u32,
    //     2 => out_func = toggle_led as u32,
    //     _ => {},
    // }

    // syscall(out_func, arg as u32);
}

#[naked]
extern "C" fn syscall(_func_ptr: u32, _arg: u32) {
    unsafe {
        asm!(
            "mov r3, #16777216",             // xPSR 1 << 24
            "str r3, [sp, #-4]!",            // xPSR
            
            "str r0, [sp, #-4]!",            // ret address (to new func) 

            "mov lr, {exc_return}",         // lr 0xFFFFFFF9
            "str lr, [sp, #-4]!",           // lr
            
            // "mov r3, #0",
            "str r1, [sp, #-4]!",
            "str r1, [sp, #-4]!",
            "str r1, [sp, #-4]!",
            "str r1, [sp, #-4]!",
            "str r1, [sp, #-4]!",
            
            "bx lr",  
            exc_return = const (0xFFFFFFF9 as u32),
            // in("r0") arg,
            // in("r1") out_func,
            options(noreturn)
        );
    }
}

fn delay(arg: usize) {
    // if arg == 100 {
    //     Interrupt::free(|cs| {
    //         if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
    //             leds[2].on();
    //         }
    //     });
    // }

    

    // kernel::delay(arg);

    Interrupt::free(|cs| {
        if let Some(ref mut delay) = DELAY.borrow(cs).borrow_mut().deref_mut() {
            delay.delay_ms(10*arg as u16);
        }
    });
}

extern "C" fn toggle_led(arg: usize) {
    // Interrupt::free(|cs| {
    //     if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
    //        if arg == 0 {
    //         leds[1].on();
    //        }
            
    //     }
    // });

    Interrupt::free(|cs| {
        if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
            leds[arg].toggle();
        }
    });
}

#[entry]
fn main() -> ! {
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024;
    unsafe { ALLOCATOR.init(start, size)}

    kernel::init();
    // kernel::create_task(FlashBlue as u32);
    // kernel::create_task(FlashRed as u32);
    // kernel::create_task(FlashOrange as u32);
    // kernel::create_task(FlashGreen as u32);
    // kernel::start();

    let tasks_bin_names = ["flash_blue", "flash_green"];


    let task_bin_blue: &[u8] = include_bytes_align_as!(f64, "flash_blue");
    let task_bin_green: &[u8] = include_bytes_align_as!(f64, "flash_green");
    let task_bins = [task_bin_blue, task_bin_green];

    for i in 0..2 {
        let task_bin = task_bins[i];
        let offset: u32 = task_bin.as_ptr() as u32;
        let file = object::File::parse(task_bin).unwrap();
        let mut task_addr: u32 = 0;
        let mut sym_addr: u32 = 0;

        for symbol in file.symbols() {
            if symbol.name() == Ok(tasks_bin_names[i]) {
                sym_addr = symbol.address() as u32;
                break;
            }
        }

        for segment in file.segments() {
            let range = segment.file_range();
            let seg_addr: u32 = segment.address() as u32;
            if sym_addr >= seg_addr {
                task_addr = range.0 as u32;
                task_addr += offset;
                task_addr += sym_addr - seg_addr;
                break;
            }
        }

        kernel::create_task(task_addr); 
    }
    
    
    
    
    kernel::start();

    loop { continue; }
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {
        Interrupt::free(|cs| {
            if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
                leds[2].on();
            }
        });
    }
}


// let task_bin_blue: &[u8] = include_bytes_align_as!(f64, "flash_blue");
//     let offset: u32 = task_bin_blue.as_ptr() as u32;
//     let file = object::File::parse(task_bin_blue).unwrap();
//     let mut task_addr: u32 = 0;
//     let mut sym_addr: u32 = 0;

//     for symbol in file.symbols() {
//         if symbol.name() == Ok("flash_blue") {
//             sym_addr = symbol.address() as u32;
//             break;
//         }
//     }

//     for segment in file.segments() {
//         let range = segment.file_range();
//         let seg_addr: u32 = segment.address() as u32;
//         if sym_addr >= seg_addr {
//             task_addr = range.0 as u32;
//             task_addr += offset;
//             task_addr += sym_addr - seg_addr;
//             break;
//         }
//     }

//     kernel::create_task(task_addr);

// #[allow(non_snake_case)]
// extern "C" fn FlashBlue() {
//     loop {
//         Interrupt::free(|cs| {
//             if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
//                 leds[3].toggle();
//             }
//         });
//         kernel::delay(100);
//     }
// }

// #[allow(non_snake_case)]
// extern "C" fn FlashRed() {
//     loop {
//         Interrupt::free(|cs| {
//             if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
//                 leds[2].toggle();
//             }
//         });
//         kernel::delay(100);
//     }
// }

// #[allow(non_snake_case)]
// extern "C" fn FlashOrange() {
//     loop {
//         Interrupt::free(|cs| {
//             if let Some(ref mut leds) = LEDS.borrow(cs).borrow_mut().deref_mut() {
//                 leds[1].toggle();
//             }
//         });
//         kernel::delay(100);
//     }
// }

#[allow(non_snake_case)]
extern "C" fn FlashGreen() {
    loop {
        unsafe {
            asm!(
                // toggle blue LED
                "mov r0, #2",
                "mov r1, #0",
                "svc #0",

                // delay for 100
                // "mov r0, #1",   
                // "mov r1, #100",
                // "svc #0",
            );
        }
        // kernel::delay(100);

        unsafe {
            asm!(
                // delay for 100
                "mov r0, #1",   
                "mov r1, #100",
                "svc #0",
            );
        }
    }
}
