
let wasm_bindgen_version = "0.2.121"

async fn command_stdout(
  command : String,
  args : Array[String],
  cwd : String,
) -> String? {
  let (exit_code, stdout) = @process.collect_stdout(command, args, cwd=cwd)
  if exit_code != 0 {
    return None
  }
  let body = stdout.text().trim()
  if body == "" {
    return None
  }
  Some(body.to_owned())
}

fn workspace_root() -> String {
  @tool.resolve_path(@env.current_dir().unwrap_or("."))
}

fn wasm_pack_cache(root : String) -> String {
  match @env.get_env_var("WASM_PACK_CACHE") {
    Some(value) => @tool.resolve_path(value)
    None => @tool.join_path(root, ".cache/wasm-pack")
  }
}

fn wasm_bindgen_root(root : String) -> String {
  @tool.join_path(wasm_pack_cache(root), ".wasm-bindgen-cargo-install-\{wasm_bindgen_version}")
}

fn wasm_bindgen_binary(root : String) -> String {
  @tool.join_path(wasm_bindgen_root(root), "bin/wasm-bindgen")
}

fn wasm_artifact(root : String) -> String {
  @tool.join_path(root, "target/wasm32-unknown-unknown/release/vize_vitrine.wasm")
}

async fn resolve_macos_sdk_root(root : String) -> String? {
  match @env.get_env_var("SDKROOT") {
    Some(value) if value.trim() != "" => return Some(@tool.resolve_path(value))
    _ => ()
  }

  for args in [
    ["--sdk", "macosx", "--show-sdk-path"],
    ["--show-sdk-path"],
  ] {
    match command_stdout("xcrun", args, root) {
      Some(value) => return Some(value)
      None => ()
    }
  }

  for candidate in [
    "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk",
    "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk",
  ] {
    if @tool.path_exists(candidate) {
      return Some(candidate)
    }
  }

  None
}

fn macos_library_path(sdk_root : String) -> String {
  let sdk_library_path = @tool.join_path(sdk_root, "usr/lib")
  match @env.get_env_var("LIBRARY_PATH") {
    Some(value) if value != "" => sdk_library_path + ":" + value
    _ => sdk_library_path
  }
}

fn wasm_rustflags() -> String {
  let flag = "--cfg getrandom_backend=\"wasm_js\""
  match @env.get_env_var("RUSTFLAGS") {
    Some(value) if value.trim() != "" => value + " " + flag
    _ => flag
  }
}

fn args_include_wasm_target(args : Array[String]) -> Bool {
  for arg in args {
    if arg == "wasm32-unknown-unknown" {
      return true
    }
  }
  false
}

async fn run_cargo(root : String, args : Array[String]) -> Int {
  let extra_env : Map[String, String] = Map([])
  if args_include_wasm_target(args) {
    extra_env["RUSTFLAGS"] = wasm_rustflags()
  }
  match resolve_macos_sdk_root(root) {
    Some(sdk_root) => {
      extra_env["SDKROOT"] = sdk_root
      extra_env["LIBRARY_PATH"] = macos_library_path(sdk_root)
      extra_env["CC"] = "clang"
      extra_env["CXX"] = "clang++"
    }
    None => ()
  }
  @process.run("cargo", args, cwd=root, extra_env=extra_env)
}

async fn ensure_directory(path : String) -> Unit {
  if @tool.path_exists(path) {
    return
  }
  @afs.mkdir(path, recursive=true, permission=0o755)
}

async fn ensure_wasm_bindgen(root : String) -> Int {
  let binary = wasm_bindgen_binary(root)
  if @tool.path_exists(binary) {
    return 0
  }

  ensure_directory(wasm_pack_cache(root))
  run_cargo(
    root,
    [
      "install",
      "--force",
      "wasm-bindgen-cli",
      "--version",
      wasm_bindgen_version,
      "--root",
      wasm_bindgen_root(root),
    ],
  )
}

async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.length() != 2 {
    println(
      "Usage: moon run --target native tools/moon/cmd/build_vitrine_wasm -- <target> <out-dir>",
    )
    return 1
  }

  let root = workspace_root()
  let target = args[0]
  let out_dir = @tool.join_path(root, args[1])

  ensure_directory(out_dir)

  let bindgen_exit = ensure_wasm_bindgen(root)
  if bindgen_exit != 0 {
    return bindgen_exit
  }

  let cargo_exit = run_cargo(
    root,
    [
      "build",
      "--manifest-path",
      @tool.join_path(root, "Cargo.toml"),
      "--release",
      "-p",
      "vize_vitrine",
      "--no-default-features",
      "--features",
      "wasm",
      "--target",
      "wasm32-unknown-unknown",
    ],
  )
  if cargo_exit != 0 {
    return cargo_exit
  }

  @process.run(
    wasm_bindgen_binary(root),
    [
      wasm_artifact(root),
      "--out-dir",
      out_dir,
      "--target",
      target,
    ],
    cwd=root,
  )
}

async fn main {
  @sys.exit(run_app())
}
