#![no_std]
#![no_main]

use rustngo_lib::*;

#[no_mangle]
fn game() {
    display_print("Cool Game 1337");

    loop {
        if is_pressed('A') {
            console_print("Button A is pressed :)");
            play_tone(440);
        } else {
            console_print("Button A is not pressed :(");
            play_tone(0);
        }

        delay(1000);
    }
}
