#![no_std]

use panic_probe as _;
mod reset;

/* These are library functions which serve as wrappers around a syscall.
 * This is how games can make use of functionality provided by firmware
 * such as hardware access.
 */
pub fn console_print(msg: &str) {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 =
        unsafe { core::mem::transmute(0x0800FC01 as *const ()) };
    syscall(1, msg.as_bytes());
}

pub fn delay(ms: u32) {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 =
        unsafe { core::mem::transmute(0x0800FC01 as *const ()) };
    syscall(2, &ms.to_le_bytes());
}

pub fn is_pressed(btn: char) -> bool {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 =
        unsafe { core::mem::transmute(0x0800FC01 as *const ()) };
    syscall(3, &[btn as u8]) == 1
}

pub fn play_tone(tone: u32) {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 =
        unsafe { core::mem::transmute(0x0800FC01 as *const ()) };
    syscall(4, &tone.to_le_bytes());
}

pub fn display_print(msg: &str) {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 =
        unsafe { core::mem::transmute(0x0800FC01 as *const ()) };
    syscall(5, msg.as_bytes());
}
