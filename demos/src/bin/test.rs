#![no_std]
#![no_main]

use rustngo_lib::*;

#[no_mangle]
fn game() {
    print("Welcome to my really cool game :D");

    loop {
        print("Hack the planet!");
        delay(1000);
    }
}
