#![no_std]
#![no_main]

use core::ptr::{addr_of, addr_of_mut};
use panic_probe as _;

// These are symbols defined by the linker script
extern "C" {
    static mut __sbss: u32;
    static mut __ebss: u32;
    static mut __sdata: u32;
    static mut __edata: u32;
    static mut __sidata: u32;
    fn game();
}

/* We add a custom Reset handler because we don't want to generate the vector tables or other
 * low-level system setup again, but still need to initialize any global variables for the loaded
 * game. This is the first thing to get executed when the game code is loaded and executed.
 * It might be undefined behavior doing this in Rust (and will need to do it in ASM),
 * but fine for now so we'll see if any issues arise.
 */
#[link_section = ".Reset"]
#[no_mangle]
extern "C" fn Reset() {
    unsafe {
        let bss_start = addr_of_mut!(__sbss) as *mut u32;
        let bss_end = addr_of_mut!(__ebss) as *mut u32;
        let bss_size = bss_end as usize - bss_start as usize;

        for i in 0..bss_size {
            *bss_start.offset(i as isize) = 0;
        }

        let flash_data = addr_of!(__sidata) as *const u32;
        let data_start = addr_of_mut!(__sdata) as *mut u32;
        let data_end = addr_of_mut!(__edata) as *mut u32;
        let data_size = data_end as usize - data_start as usize;

        for i in 0..data_size {
            *data_start.offset(i as isize) = *flash_data.offset(i as isize);
        }

        // The game code hopefully defined this function, which serves as its entry point
        game();
    }
}

/* These are library functions which serve as wrappers around a syscall.
 * This is how games can make use of functionality provided by firmware
 * such as hardware access.
 */
pub fn print(msg: &str) {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 = unsafe { core::mem::transmute(0x0800FC01) };
    syscall(1, msg.as_bytes());
}

pub fn delay(ms: u32) {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 = unsafe { core::mem::transmute(0x0800FC01) };
    syscall(2, &ms.to_le_bytes());
}
