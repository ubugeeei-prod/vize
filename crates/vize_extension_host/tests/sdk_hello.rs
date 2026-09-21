//! P6-2 acceptance: a hello-world input dialect builds from the **packed**
//! SDK alone and runs through the host.
//!
//! The test packs `vize_extension_sdk` (`cargo package`), unpacks the
//! tarball outside the source tree, copies the example's sources
//! (`crates/vize_extension_sdk/examples/hello-dialect`) next to it with a
//! manifest whose only vize dependency is that unpacked tarball — no path into
//! the workspace — builds the component for `wasm32-wasip2`, and exchanges a
//! block in both hosting modes with exact pages.

use std::path::{Path, PathBuf};
use std::process::Command;

use vize_extension_host::outproc::{OutOfProcessGuest, serve_command};
use vize_extension_host::wasm::WasmGuest;
use vize_extension_host::{InputDialectGuest, Session, SourceBlock};
use vize_s0::{String, cstr};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cargo(args: &[&str], extra: &[&Path]) {
    let mut command = Command::new(env!("CARGO"));
    command.args(args).args(extra);
    for leaked in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_TARGET",
        "CARGO_TARGET_DIR",
    ] {
        command.env_remove(leaked);
    }
    let status = command.status().expect("cargo runs");
    assert!(status.success(), "cargo {args:?} failed: {status}");
}

/// Pack the SDK, unpack it, and build the example against the tarball.
fn build_hello() -> PathBuf {
    let target_root = std::env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| root().join("target"), PathBuf::from)
        .join("contract-guests/sdk-hello");
    let _ = std::fs::remove_dir_all(&target_root);
    let pack = target_root.join("pack");
    cargo(
        &[
            "package",
            "-p",
            "vize_extension_sdk",
            "--allow-dirty",
            "--no-verify",
        ],
        &[
            Path::new("--manifest-path"),
            &root().join("Cargo.toml"),
            Path::new("--target-dir"),
            &pack,
        ],
    );
    let version = env!("CARGO_PKG_VERSION");
    let crate_file = pack.join(cstr!("package/vize_extension_sdk-{version}.crate").as_str());
    let unpacked = target_root.join("sdk");
    std::fs::create_dir_all(&unpacked).expect("creates the unpack directory");
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&crate_file)
        .arg("-C")
        .arg(&unpacked)
        .status()
        .expect("tar runs");
    assert!(
        status.success(),
        "unpacking {} failed",
        crate_file.display()
    );
    let sdk = unpacked.join(cstr!("vize_extension_sdk-{version}").as_str());

    let example = root().join("crates/vize_extension_sdk/examples/hello-dialect");
    let project = target_root.join("project");
    std::fs::create_dir_all(project.join("src")).expect("creates the project");
    std::fs::copy(example.join("src/lib.rs"), project.join("src/lib.rs")).expect("copies lib.rs");
    let manifest = std::fs::read_to_string(example.join("Cargo.toml")).expect("reads the manifest");
    let local = "vize_extension_sdk = { path = \"../..\" }";
    assert_eq!(
        manifest.matches(local).count(),
        1,
        "the example has one local SDK edge"
    );
    let packed = cstr!(
        "vize_extension_sdk = {{ path = {:?} }}",
        sdk.to_str().expect("a UTF-8 target path")
    );
    std::fs::write(project.join("Cargo.toml"), manifest.replace(local, &packed))
        .expect("writes the manifest");
    cargo(
        &["build", "--release", "--target", "wasm32-wasip2"],
        &[
            Path::new("--manifest-path"),
            &project.join("Cargo.toml"),
            Path::new("--target-dir"),
            &target_root.join("target"),
        ],
    );
    target_root.join("target/wasm32-wasip2/release/hello_dialect.wasm")
}

const S1: &str =
    "[s1]\nbytes=14\n\n[s1.tree]\ntext 0:0:4\ntext 4:4:10\ntext 10:10:11\ntext 11:11:14\n\n";
const S2: &str = "[disegno]\nops=7\n\n[disegno.ops]\nui.element ul @40:54\n  ui.element li @40:43\n    \
                  ui.text \"Hello, Ada!\" @40:43\n  ui.element li @44:49\n    ui.text \"Hello, Grace!\" @44:49\n  \
                  ui.element li @51:54\n    ui.text \"Hello, Lin!\" @51:54\n\n";

#[test]
fn a_hello_dialect_built_from_the_packed_sdk_runs_in_both_modes() {
    let component = build_hello();
    let runner = Path::new(env!("CARGO_BIN_EXE_vize-extension-host"));
    let guests: [(&str, Box<dyn InputDialectGuest>); 2] = [
        (
            "out-of-process",
            Box::new(OutOfProcessGuest::spawn(serve_command(runner, &component)).expect("loads")),
        ),
        (
            "in-process",
            Box::new(WasmGuest::load(&component).expect("loads")),
        ),
    ];
    let block = SourceBlock {
        source: String::from("Ada\nGrace\n\nLin"),
        base: 40,
        lang: Some(String::from("hello")),
    };
    for (mode, guest) in guests {
        let mut session = Session::open(guest).unwrap_or_else(|error| panic!("{mode}: {error}"));
        assert_eq!(
            session.negotiated().langs,
            [String::from("hello")],
            "{mode}"
        );
        let accepted = session
            .lower_block(&block)
            .unwrap_or_else(|error| panic!("{mode}: {error}"));
        assert_eq!(accepted.lowered.surface.text, S1, "{mode}");
        assert_eq!(accepted.lowered.semantic.text, S2, "{mode}");
        assert_eq!(accepted.lowered.diagnostics, [], "{mode}");
    }
}
