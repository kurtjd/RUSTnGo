#![no_std]

use panic_halt as _;
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

pub fn display_print(x: u8, y: u8, msg: &str) {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 =
        unsafe { core::mem::transmute(0x0800FC01 as *const ()) };
    let mut buf: [u8; 22] = [0; 22];
    buf[0] = x;
    buf[1] = y;
    buf[2..msg.len() + 2].copy_from_slice(msg.as_bytes());
    syscall(5, &buf[..msg.len() + 2]);
}

pub fn display_draw_rect(x: u8, y: u8, w: u8, h: u8, fill: bool) {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 =
        unsafe { core::mem::transmute(0x0800FC01 as *const ()) };
    syscall(6, &[x, y, w, h, fill as u8]);
}

pub fn display_draw_circle(x: u8, y: u8, d: u8, fill: bool) {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 =
        unsafe { core::mem::transmute(0x0800FC01 as *const ()) };
    syscall(7, &[x, y, d, fill as u8]);
}

pub fn display_update() {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 =
        unsafe { core::mem::transmute(0x0800FC01 as *const ()) };
    syscall(8, &[]);
}

pub fn display_clear() {
    let syscall: extern "C" fn(u8, &[u8]) -> u8 =
        unsafe { core::mem::transmute(0x0800FC01 as *const ()) };
    syscall(9, &[]);
}
