#![no_std]
#![no_main]

use rustngo_lib::*;

#[no_mangle]
fn game() {
    print("Welcome to my really cool game :D");

    loop {
        if is_pressed('A') {
            print("Button A is pressed :)");
        } else {
            print("Button A is not pressed :(");
        }

        delay(1000);
    }
}
