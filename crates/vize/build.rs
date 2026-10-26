//! Linux x64 release links of the CLI fold identical code and pack relative
//! relocations: 4.8 MiB of the TS-44 server binary (#10817).

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // `--icf=all` needs lld, rustc's default linker for this target, so a
    // configured linker (Wild in most CI jobs) keeps its defaults. lld still
    // keeps address-significant data unique; function addresses never were.
    let default_lld = env::var("TARGET").is_ok_and(|target| target == "x86_64-unknown-linux-gnu")
        && env::var_os("RUSTC_LINKER").is_none();
    if default_lld && env::var("PROFILE").is_ok_and(|profile| profile == "release") {
        println!("cargo:rustc-link-arg-bins=-Wl,--icf=all");
        println!("cargo:rustc-link-arg-bins=-Wl,-z,pack-relative-relocs");
    }
}
