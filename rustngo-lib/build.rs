use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put `link.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("link.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `link.x`
    // here, we ensure the build script is only re-run when
    // `link.x` is changed.
    println!("cargo:rerun-if-changed=link.x");

    // Specify linker arguments.

    // `--nmagic` is required if memory section addresses are not aligned to 0x10000,
    // for example the FLASH and RAM sections in your `memory.x`.
    // See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
    // println!("cargo:rustc-link-arg=--nmagic");
    // println!("cargo:rustc-link-arg=-Tdefmt.x");

    // Set the linker script to the one provided by cortex-m-rt.
    // println!("cargo:rustc-link-arg=-Tlink.x");
}
