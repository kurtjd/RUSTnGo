#![no_std]
#![no_main]

use panic_probe as _;

extern "C" {
    static mut __sbss: u32;
    static mut __ebss: u32;
    static mut __sdata: u32;
    static mut __edata: u32;
    static mut __sidata: u32;
    fn game();
}

#[link_section = ".Reset"]
#[no_mangle]
pub extern "C" fn Reset() {
    unsafe {
        let bss_start = &mut __sbss as *mut u32;
        let bss_end = &mut __ebss as *mut u32;
        let bss_size = bss_end as usize - bss_start as usize;
        for i in 0..bss_size {
            *bss_start.offset(i as isize) = 0;
        }
    }

    unsafe {
        let data_start = &__sdata as *const u32;
        let data_end = &__edata as *const u32;
        let data_size = data_end as usize - data_start as usize;
        let flash_data = &__sidata as *const u32;
        let ram_data = data_start as *mut u32;
        for i in 0..data_size {
            *ram_data.offset(i as isize) = *flash_data.offset(i as isize);
        }
    }

    unsafe { game() };
}

#[no_mangle]
pub fn make_syscall() {
    let syscall_raw: extern "C" fn() = unsafe { core::mem::transmute(0x0800FC01) };
    syscall_raw();
}
