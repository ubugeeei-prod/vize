/// Builds the WebAssembly package used for npm publication.
/// The release workflow treats `npm/wasm` as a generated artifact. This
/// script encapsulates the exact build steps needed to produce that artifact:
/// build the full browser and focused Cloudflare Workers artifacts for
/// `wasm32-unknown-unknown` and run `wasm-bindgen` for both entry points. The
/// generated files are written alongside the checked-in package metadata.

fn wasm_rustflags() -> String {
  let flag = "--cfg getrandom_backend=\"wasm_js\""
  match @env.get_env_var("RUSTFLAGS") {
    Some(value) if value.trim() != "" => value + " " + flag
    _ => flag
  }
}

/// Main entry point for generating the publishable WASM package directory.
async fn run_app() -> Int {
  let cargo_exit = @process.run(
    "cargo",
    [
      "build",
      "--release",
      "-p",
      "vize_vitrine",
      "--no-default-features",
      "--features",
      "wasm",
      "--target",
      "wasm32-unknown-unknown",
    ],
    extra_env={"RUSTFLAGS": wasm_rustflags()},
  )
  if cargo_exit != 0 {
    println("Failed to build the WASM crate")
    return 1
  }

  let bindgen_exit = @process.run(
    "wasm-bindgen",
    [
      "target/wasm32-unknown-unknown/release/vize_vitrine.wasm",
      "--out-dir",
      "npm/wasm",
      "--target",
      "web",
    ],
  )
  if bindgen_exit != 0 {
    println("Failed to run wasm-bindgen")
    return 1
  }

  let workerd_cargo_exit = @process.run(
    "cargo",
    [
      "build",
      "--release",
      "-p",
      "vize_vitrine",
      "--no-default-features",
      "--features",
      "workerd",
      "--target",
      "wasm32-unknown-unknown",
    ],
    extra_env={"RUSTFLAGS": wasm_rustflags()},
  )
  if workerd_cargo_exit != 0 {
    println("Failed to build the workerd WASM crate")
    return 1
  }

  let workerd_bindgen_exit = @process.run(
    "wasm-bindgen",
    [
      "target/wasm32-unknown-unknown/release/vize_vitrine.wasm",
      "--out-dir",
      "npm/wasm",
      "--out-name",
      "vize_workerd",
      "--target",
      "web",
    ],
  )
  if workerd_bindgen_exit != 0 {
    println("Failed to run wasm-bindgen for workerd")
    return 1
  }

  0
}

async fn main {
  @sys.exit(run_app())
}
