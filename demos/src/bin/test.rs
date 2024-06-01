#![no_std]
#![no_main]

use rustngo_lib::make_syscall;

#[no_mangle]
fn game() {
    make_syscall();
}
