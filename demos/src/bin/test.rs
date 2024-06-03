#![no_std]
#![no_main]

use rustngo_lib::*;

#[no_mangle]
fn game() {
    let mut x: u8 = 10;
    let y: u8 = 20;
    let d = 5;
    let mut won = false;

    console_print("Welcome to my really cool game :D");
    loop {
        if is_pressed('A') {
            x += 1;
        }
        if x >= 100 && !won {
            won = true;
            play_tone(440);
        }

        display_clear();
        display_draw_circle(x, y, d, true);
        if !won {
            display_print(10, 10, "Cool Game");
        } else {
            display_print(10, 10, "You Won :D");
        }
        display_update();

        delay(100);
    }
}
