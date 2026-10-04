"""Reject ambient build overrides; record pinned original compiler inputs."""

import json
import os
import subprocess
from pathlib import Path
from common import command, git, require, sha256, write_json


def compiler_envelope(root, worktree, original_receipt, output):
    # Python 3.12 on the pinned Ubuntu runner; no extra parser dependency.
    import tomllib
    compiler = command(["rustc", "-Vv"])
    require(compiler == original_receipt["rustcVersion"], "exact original rustc -Vv identity")
    forbidden = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_TARGET_DIR", "CARGO_BUILD_TARGET",
                 "CARGO_BUILD_TARGET_DIR", "CARGO_BUILD_BUILD_DIR",
                 "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTFLAGS",
                 "CARGO_BUILD_RUSTC", "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER")
    require(all(key not in os.environ for key in forbidden), "inherited compiler/target override")
    require(not any(key.startswith(("CARGO_PROFILE_", "CARGO_TARGET_")) for key in os.environ),
            "inherited profile/target linker override")
    require(os.environ.get("CARGO_BUILD_JOBS") == "12" and
            os.environ.get("CARGO_INCREMENTAL") == "0", "original jobs/incremental envelope")
    cargo_home = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")))
    home_path = cargo_home / "config.toml"
    home = tomllib.loads(home_path.read_text())
    require(not any(key in home for key in ("build", "profile", "unstable", "env")),
            "unreviewed home build/profile/config override")
    require(not (cargo_home / "config").exists(), "unreviewed alternate Cargo home config")
    wild = Path(os.environ["RUNNER_TEMP"]) / "wild-install/wild"
    targets = home.get("target", {})
    require(set(targets) == {"x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl"},
            "only pinned wild action target configs")
    for config in targets.values():
        require(config == {"linker": "clang", "rustflags": [f"-Clink-arg=--ld-path={wild}"]},
                "exact pinned original wild compiler flags")
    require("0.9.0" in command([str(wild), "--version"]), "pinned actual linker binary")
    for path in (".cargo/config.toml", "Cargo.toml", "Cargo.lock", ".config/nextest.toml"):
        require(git(worktree, "rev-parse", f"HEAD:{path}") == git(root, "rev-parse", f"HEAD:{path}"),
                "frozen original compiler profile/lock/config must match diagnostic source")
    write_json(output / "compiler-envelope.json", {
        "rustc": compiler, "cargo": command(["cargo", "-Vv"]),
        "nextest": command(["cargo", "nextest", "--version"]),
        "effective_target_configs": targets, "wild_sha256": sha256(wild),
        "wild_version": command([str(wild), "--version"]),
        "overrides_rejected": list(forbidden),
        "original_flag_source": "same pinned wild action at 0bbbfa5df4380cab8e63cb8505a1ce65e1d10203",
        "environment": {key: os.environ.get(key) for key in
                        ("CARGO_HOME", "CARGO_BUILD_JOBS", "CARGO_INCREMENTAL", "RUSTUP_TOOLCHAIN",
                         "VIZE_TEST_REQUIRE_TSGO", "VIZE_NUXT_CONFIG_ITERATIONS")}})


def elf_dependencies(binary, output):
    env = os.environ.copy()
    env["LD_LIBRARY_PATH"] = str(binary.parent) + ":" + env.get("LD_LIBRARY_PATH", "")
    result = subprocess.run(["ldd", str(binary)], env=env, capture_output=True, text=True)
    raw = result.stdout + result.stderr
    write_json(output, {"binary_sha256": sha256(binary), "command": ["ldd", str(binary)],
                       "exit_code": result.returncode, "raw": raw,
                       "ld_library_path": env["LD_LIBRARY_PATH"]})
    require("not found" not in raw and
            (result.returncode == 0 or "statically linked" in raw or "not a dynamic executable" in raw),
            "selected actual ELF dependency resolution must succeed")
