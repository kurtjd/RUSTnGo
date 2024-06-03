use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Copy test game binary to our folder for now
    let status = Command::new("arm-none-eabi-objcopy")
        .args([
            "-O",
            "binary",
            "../target/thumbv7m-none-eabi/release/pong",
            "game.bin",
        ])
        .status()
        .expect("Failed to copy test game binary");
    if !status.success() {
        panic!("Failed to copy test game binary");
    }

    /* Put `link.x` in our output directory and ensure it's
     * on the linker search path.
     */
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("link.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=link.x");

    // Specify linker arguments.
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    println!("cargo:rustc-link-arg=-Tlink.x");
}
