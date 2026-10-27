//! Linux x64 release links of the CLI fold identical code and pack relative
//! relocations: 4.8 MiB of the TS-44 server binary (#10817).

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let target = env::var("TARGET").is_ok_and(|target| target == "x86_64-unknown-linux-gnu");
    let release = env::var("PROFILE").is_ok_and(|profile| profile == "release");
    if target && release && default_linker() {
        println!("cargo:rustc-link-arg-bins=-Wl,--icf=all");
        println!("cargo:rustc-link-arg-bins=-Wl,-z,pack-relative-relocs");
    }
}

/// `--icf=all` needs lld, rustc's default linker for this target, so any
/// linker chosen in Cargo config or rustc flags (Wild in most CI jobs) keeps
/// its defaults. lld still keeps address-significant data unique; function
/// addresses never were.
fn default_linker() -> bool {
    let flags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    env::var_os("RUSTC_LINKER").is_none()
        && !["linker", "link-self-contained", "-fuse-ld", "--ld-path"]
            .iter()
            .any(|flag| flags.contains(flag))
}
