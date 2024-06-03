use defmt::*;

pub fn load_and_exec() {
    /* For now, load the game binary directly at compile-time
     * (goal is to load at runtime from SD card) into RAM.
     * Also want to experiment with loading into Flash eventually.
     * Once binary is loaded, attempt to branch to it.
     */
    let game_mem: *mut u8 = 0x20001800 as *mut u8;
    let game_bin = include_bytes!("../game.bin");
    unsafe {
        game_mem.copy_from(game_bin.as_ptr(), game_bin.len());
    }

    /* The Rust compiler generates a blx instruction here (as opposed to bl),
     * which changes between ARM and Thumb depending on the LSB (if 0, branch and change to
     * ARM mode). This is annoying because all of our code is generated in Thumb mode, so have to
     * make sure to call with LSB set to stay in Thumb mode. There is likely a Rustier alternative
     * since transmute is very unsafe and I'm probably just hitting undefined behavior.
     */
    info!("Starting game...");
    let game: extern "C" fn() = unsafe { core::mem::transmute(0x20001801 as *const ()) };
    game();
}