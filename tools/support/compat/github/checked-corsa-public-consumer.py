#!/usr/bin/env python3
"""Optional post-publication acceptance. Never publishes or registers packages."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import tomllib

from checked_corsa_public_contract import (
    PUBLIC, archive_evidence, base, check_outside_workspace, public_environment,
    validate_identity, verify_public_graph,
)


class PublicAcceptance(base.Acceptance):
    def consumer(self, name, manifest, source):
        parsed = tomllib.loads(manifest)
        base.require(not {"patch", "replace"} & set(parsed), "public consumers cannot replace dependencies")
        base.require(all(set(dependency) <= {"package", "version"} for dependency in parsed["dependencies"].values()),
                     "public consumers require version-only registry dependencies")
        base.require(all(dependency.get("version") == "=" + self.version for dependency in parsed["dependencies"].values()),
                     "public consumer dependencies must match the explicit exact version")
        directory = self.output / name
        (directory / "src").mkdir(parents=True)
        (directory / "Cargo.toml").write_text(base.isolated_manifest(manifest, name))
        (directory / "src/lib.rs").write_bytes(source)
        return directory

    def metadata(self, label, directory, features=(), locked=False):
        flags = ["--locked"] if locked else []
        # Only metadata resolves/downloads dependencies; builds use --locked --offline.
        return json.loads(self.command(label, ["cargo", "metadata", "--format-version", "1", *flags, *features], directory))


def run(root, output, version, source_sha):
    validate_identity(version, source_sha)
    check_outside_workspace(output, root)
    base.require(not output.exists(), "public evidence directory must be new")
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    status = subprocess.check_output(["git", "status", "--porcelain"], cwd=root, text=True)
    workspace = tomllib.loads((root / "Cargo.toml").read_text())
    base.require(head == source_sha and not status and workspace["workspace"]["package"]["version"] == version,
                 "run public acceptance from the clean exact release checkout")
    output.mkdir(parents=True)
    home, work = output / "cargo-home", output / "work"
    home.mkdir()
    work.mkdir()
    isolated = public_environment(dict(os.environ), home, output / "target")
    isolated["RUSTUP_TOOLCHAIN"] = tomllib.loads((root / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    os.environ.clear()
    os.environ.update(isolated)
    runner = PublicAcceptance(work, output / "evidence", source_sha)
    runner.version = version
    runner.command("cargo-version", ["cargo", "-Vv"], work)
    runner.command("rustc-version", ["rustc", "-Vv"], work)
    fixture = root / "tests/_fixtures/ci/checked-corsa-consumer"
    template = (fixture / "Cargo.toml").read_text().replace('version = "*"', 'version = "=' + version + '"')
    consumer = runner.consumer("public-six", template, (fixture / "src/lib.rs").read_bytes())
    graphs, metadata_values = [], []
    for mode, features in [("default", []), ("distributed", ["--features", "distributed"])]:
        metadata = runner.metadata("public-" + mode + "-metadata", consumer, features)
        graphs.append({"mode": mode, **verify_public_graph(metadata, version, home, consumer, bool(features)),
                       "lockPackages": tomllib.loads((consumer / "Cargo.lock").read_text())["package"]})
        metadata_values.append((metadata, consumer))
    runner.builds("public-six", consumer)

    canon_manifest = '[package]\nname = "public-canon-consumer"\nversion = "0.0.0"\nedition = "2024"\n[workspace]\n'
    canon_manifest += '[dependencies]\nvize_canon = { version = "=' + version + '" }\n'
    canon = runner.consumer("public-canon-default", canon_manifest, (root / "tests/_fixtures/ci/checked-canon-consumer/lib.rs").read_bytes())
    metadata = runner.metadata("public-canon-default-metadata", canon)
    graphs.append({"mode": "canon-default-native", **verify_public_graph(metadata, version, home, canon, False, canon=True),
                   "lockPackages": tomllib.loads((canon / "Cargo.lock").read_text())["package"]})
    metadata_values.append((metadata, canon))
    runner.build_once("public-canon", "default-native", canon)

    archives = {}
    for metadata, directory in metadata_values:
        lock = tomllib.loads((directory / "Cargo.lock").read_text())
        for package in metadata["packages"]:
            if package["name"] not in PUBLIC or package["name"] in archives:
                continue
            source = root / "crates/vize_canon" if package["name"] == "vize_canon" else root / "vendor" / package["name"].removeprefix("vize_")
            archives[package["name"]] = archive_evidence(package, home, lock, source_sha, source)
    base.require(set(archives) == PUBLIC, "public archive evidence must cover all seven owned packages")
    base.require(not any((home / name).exists() for name in ["config", "config.toml"]),
                 "fresh Cargo home acquired dependency source configuration")
    for directory in [home, work, runner.output, consumer, canon]:
        base.require(not any((directory / ".cargo" / name).exists() for name in ["config", "config.toml"]),
                     "Cargo configuration appeared during public acceptance")
    base.require(runner.command("release-final-head", ["git", "rev-parse", "HEAD"], root).strip() == source_sha
                 and not runner.command("release-final-status", ["git", "status", "--porcelain"], root),
                 "release checkout changed during public acceptance")
    receipt = {"sourceSha": source_sha, "version": version, "commands": runner.commands,
               "consumerBuilds": runner.consumer_builds, "graphs": graphs, "archives": list(archives.values()),
               "publicRegistryAcceptance": True, "sixDefaultAndDistributedCompiledAndTested": True,
               "canonDefaultNativeCompiled": True, "cliVariantsCompiled": False, "cliPackagePublishable": False,
               "freshCargoHome": str(home), "sourcePatches": False, "sourceReplacement": False}
    (runner.output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    args = parser.parse_args()
    run(args.root.resolve(), args.output.resolve(), args.version, args.source_sha)
