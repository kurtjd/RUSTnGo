#![no_std]
#![no_main]

use rustngo_lib::*;

#[no_mangle]
fn game() {
    print("Welcome to my really cool game :D");

    loop {
        if is_pressed('A') {
            print("Button A is pressed :)");
            play_tone(440);
        } else {
            print("Button A is not pressed :(");
            play_tone(0);
        }

        delay(1000);
    }
}
