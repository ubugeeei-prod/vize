
fn wasm_rustflags() -> String {
  let flag = "--cfg getrandom_backend=\"wasm_js\""
  match @sys.get_env_var("RUSTFLAGS") {
    Some(value) if value.trim() != "" => value + " " + flag
    _ => flag
  }
}

async fn run_app() -> Int {
  let out_dirs = @tool.cli_args()
  if out_dirs.is_empty() {
    println(
      "Usage: moon run --target native tools/moon/cmd/github/build_vitrine_wasm -- <out-dir> [out-dir...]",
    )
    return 1
  }
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
    return cargo_exit
  }
  for out_dir in out_dirs {
    let bindgen_exit = @process.run(
      "wasm-bindgen",
      [
        "target/wasm32-unknown-unknown/release/vize_vitrine.wasm",
        "--out-dir",
        out_dir,
        "--target",
        "web",
      ],
    )
    if bindgen_exit != 0 {
      return bindgen_exit
    }
  }
  0
}

async fn main {
  @sys.exit(run_app())
}
