#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
//use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// OP: 1
fn print(buf: &[u8]) {
    let msg = core::str::from_utf8(buf).unwrap();
    info!("{}", msg)
}

// OP: 2
fn delay(buf: &[u8]) {
    let start = embassy_time::Instant::now();
    loop {
        let elapsed = embassy_time::Instant::now() - start;
        let duration: [u8; 4] = buf.try_into().unwrap();
        if elapsed >= embassy_time::Duration::from_millis((u32::from_le_bytes(duration)).into()) {
            break;
        }
    }
}

/* The syscall serves as the single point of entry by games into the firmware.
 * This provides extra functionality to games such as hardware access.
 * This allows us to not have to bloat game binary sizes by copying all this functionality
 * into their binaries. Though it does make for some interesting linker challenges :)
 */
#[link_section = ".syscall"]
#[inline(never)]
#[no_mangle]
#[allow(clippy::needless_return)]
fn syscall(op: u8, buf: &[u8]) -> u8 {
    match op {
        1 => print(buf),
        2 => delay(buf),
        _ => info!("Unrecognized syscall"),
    }

    return 0;
}

/* Embassy may likely be overkill, or not the best solution,
 * but using it for now to make things easier. Can always try
 * something else in the future.
 */
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Just turning on LED for fun
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PC13, Level::High, Speed::Low);
    led.set_high();

    /* For now, load the game binary directly at compile-time
     * (goal is to load at runtime from SD card) into RAM.
     * Also want to experiment with loading into Flash eventually.
     * Once binary is loaded, attempt to branch to it.
     */
    let game_mem: *mut u8 = 0x20001800 as *mut u8;
    let game_bin = include_bytes!("../game.bin");
    unsafe { game_mem.copy_from(game_bin.as_ptr(), game_bin.len()); }

    /* The Rust compiler generates a blx instruction here (as opposed to bl),
     * which changes between ARM and Thumb depending on the LSB (if 0, branch and change to ARM mode).
     * This is annoying because all of our code is generated in Thumb mode, so have to make sure to call
     * with LSB set to stay in Thumb mode. There is likely a Rustier alternative since transmute is
     * very unsafe and I'm probably just hitting undefined behavior.
     */
    info!("Starting game...");
    let game_start: extern "C" fn() = unsafe { core::mem::transmute(0x20001801) };
    game_start();

    defmt::panic!("If you are seeing this the universe imploded!");
}
