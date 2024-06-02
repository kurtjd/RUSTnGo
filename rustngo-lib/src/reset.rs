use core::ptr::{write_bytes, copy_nonoverlapping, addr_of, addr_of_mut};
use core::mem::size_of;

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
        // Zero out .bss section
        let bss_start = addr_of_mut!(__sbss);
        let bss_end = addr_of_mut!(__ebss);
        let bss_size = bss_end as usize - bss_start as usize;
        write_bytes(bss_start, 0, bss_size / size_of::<u32>());

        // Initialize .data section by copying initial values from Flash into RAM
        let flash_start = addr_of!(__sidata);
        let data_start = addr_of_mut!(__sdata);
        let data_end = addr_of_mut!(__edata);
        let data_size = data_end as usize - data_start as usize;
        copy_nonoverlapping(flash_start, data_start, data_size / size_of::<u32>());
        
        // The game code hopefully defined this function, which serves as its entry point
        game();
    }
}