#![no_std]
#![no_main]

use rustngo_lib::*;

#[no_mangle]
fn game() {
    loop {
        print("Hack the planet!");
        delay(1000);
    }
}
