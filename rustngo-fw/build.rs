use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
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
