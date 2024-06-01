#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".syscall"]
#[inline(never)]
#[no_mangle]
pub extern "C" fn syscall() {
    info!("YOU MADE A SYSCALL");
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    let game_mem: *mut u8 = 0x20001800 as *mut u8;
    let game = include_bytes!("../game.bin");
    unsafe { game_mem.copy_from(game.as_ptr(), game.len()); }

    /* The Rust compiler generates a blx instruction, which changes between ARM and Thumb
     * depending on the LSB (if 0, branch and change to ARM mode). This is annoying because
     * all of our code is generated in Thumb mode, so have to make sure to call with LSB set
     * to stay in Thumb mode.
     */
    let game_exec: extern "C" fn() = unsafe { core::mem::transmute(0x20001801) };

    info!("Exec game...");
    game_exec();

    info!("Starting blink...");
    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
        info!("BLINK!");
    }
}
