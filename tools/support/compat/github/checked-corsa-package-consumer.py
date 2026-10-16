#!/usr/bin/env python3
"""Build independent path and Cargo-normalized archive consumers, without publishing.

Requires Python 3.11+, Cargo, a fetched workspace lock, and SOURCE_SHA. Run on the
same CI worker as the native checks to reuse CARGO_TARGET_DIR. Archive creation
uses --no-verify only because the six new registry names are not yet published;
the separate offline archive-consumer builds below provide compilation proof.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile
import time
import tomllib

from checked_corsa_source_consumers import qualify_source_consumers


ALIASES = ("corsa_core", "corsa_jsonrpc", "corsa_client", "corsa_lsp", "corsa_orchestrator", "corsa")
OWNED = {"vize_" + name for name in ALIASES}
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"
RUNTIME_VERSION = "1.14.2"
GENERATED = {"Cargo.toml", "Cargo.toml.orig", "Cargo.lock", ".cargo_vcs_info.json"}
PROFILE = ["--profile", "ci", "--config", 'profile.ci.inherits="dev"',
           "--config", "profile.ci.debug=false", "--config", "profile.ci.incremental=false",
           "--config", "profile.ci.codegen-units=256"]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(body):
    return hashlib.sha256(body).hexdigest()


def isolated_manifest(manifest, name):
    """Distinct root names prevent Cargo reusing a sibling root's cached crate."""
    require(re.fullmatch(r"[a-z0-9-]+", name), "invalid consumer identity")
    original = tomllib.loads(manifest)["package"]["name"]
    before = 'name = ' + json.dumps(original)
    require(manifest.count(before) == 1, "ambiguous consumer package name")
    return manifest.replace(before, 'name = ' + json.dumps(original + "-" + name))


def verify_consumer_build(stdout, directory):
    manifest = directory / "Cargo.toml"
    name = tomllib.loads(manifest.read_text())["package"]["name"].replace("-", "_")
    artifacts = [row for line in stdout.splitlines() if (row := json.loads(line)).get("reason") == "compiler-artifact"
                 and Path(row["manifest_path"]).resolve() == manifest.resolve() and row["target"]["kind"] == ["lib"]]
    require(len(artifacts) == 1, "build must report exactly one consumer library artifact")
    artifact = artifacts[0]
    require(artifact["fresh"] is False, "consumer root must actually compile, not reuse another cached root")
    require(artifact["target"]["name"] == name and Path(artifact["target"]["src_path"]).resolve() == (directory / "src/lib.rs").resolve(),
            "compiled consumer source identity mismatch")
    return {key: artifact[key] for key in ["package_id", "manifest_path", "target", "fresh", "features", "filenames"]}


def archive_members(archive, prefix):
    """Read regular archive members without trusting tar extraction paths."""
    bodies = {}
    with tarfile.open(archive, "r:gz") as stream:
        for member in stream:
            require(member.isfile() and member.name.startswith(prefix + "/"), "foreign/nonregular archive member")
            relative = member.name[len(prefix) + 1:]
            path = PurePosixPath(relative)
            require(relative and not path.is_absolute() and ".." not in path.parts
                    and str(path) == relative and relative not in bodies, "unsafe/duplicate archive path")
            bodies[relative] = stream.extractfile(member).read()
    return bodies


def verify_packaged_source(directory, bodies):
    """Require every source member, with only Cargo's reserved files regenerated."""
    manifest = tomllib.loads((directory / "Cargo.toml").read_text())
    exclusions = set(manifest["package"].get("exclude", []))
    require(exclusions <= GENERATED - {"Cargo.toml"}, "only reserved Cargo source files may be excluded")
    source_members = set()
    for member in directory.rglob("*"):
        require(not member.is_symlink(), "symlink in package source inventory")
        if member.is_file():
            source_members.add(member.relative_to(directory).as_posix())
    expected_source = source_members - GENERATED
    # --exclude-lockfile deliberately omits the generated package lock; the
    # archived original manifest and VCS receipt must instead be regenerated.
    expected_archive = expected_source | (GENERATED - {"Cargo.lock"})
    require("LICENSE" in bodies and "LICENSE" in source_members, "archive must preserve LICENSE")
    require(bodies["LICENSE"] == (directory / "LICENSE").read_bytes(), "archive LICENSE bytes changed")
    require(set(bodies) == expected_archive,
            "archive member inventory differs: missing=" + str(sorted(expected_archive - set(bodies)))
            + ", unexpected=" + str(sorted(set(bodies) - expected_archive)))
    for relative in expected_source:
        require(bodies[relative] == (directory / relative).read_bytes(), "archived source/readme/license changed: " + relative)
    return sorted(expected_source)


def verify_graph(metadata, version, providers, distributed):
    packages = metadata["packages"]
    owned = [package for package in packages if package["name"] in OWNED]
    require(len(owned) == len(OWNED) and {package["name"] for package in owned} == OWNED,
            "consumer must resolve exactly the six owned packages")
    require(not any(package["name"] in ALIASES for package in packages), "upstream six-package graph leaked in")
    runtime = [package for package in packages if package["name"] == "corsa_runtime"]
    require(len(runtime) == 1 and runtime[0]["version"] == RUNTIME_VERSION and runtime[0]["source"] == REGISTRY,
            f"runtime must remain the unique published {RUNTIME_VERSION} provider")
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    for package in owned:
        require(package["version"] == version, "owned package version differs from workspace")
        manifest, source = providers[package["name"]]
        require(Path(package["manifest_path"]).resolve() == manifest.resolve()
                and package["source"] == source, "unexpected consumer package provider")
        if package["name"] in {"vize_corsa", "vize_corsa_orchestrator"}:
            require(("experimental-distributed" in nodes[package["id"]]["features"]) == distributed,
                    "experimental feature activation differs from consumer request")
    return [{"name": package["name"], "version": package["version"], "source": package["source"],
             "manifest": package["manifest_path"], "features": nodes[package["id"]]["features"]}
            for package in owned]


class Acceptance:
    def __init__(self, root, output, source_sha):
        self.root, self.output, self.source_sha = root, output, source_sha
        self.commands = []
        self.consumer_builds = []
        require(Path(os.environ.get("CARGO_TARGET_DIR", "")).is_absolute(),
                "set an absolute shared CARGO_TARGET_DIR to avoid duplicate native builds")
        output.mkdir(parents=True, exist_ok=False)

    def command(self, label, args, cwd=None, status=0):
        started = time.monotonic()
        print(f"[{label}] {' '.join(map(str, args))}", flush=True)
        with (self.output / (label + ".stdout")).open("wb") as stdout, (self.output / (label + ".stderr")).open("wb") as stderr:
            result = subprocess.run(list(map(str, args)), cwd=cwd or self.root, stdout=stdout, stderr=stderr,
                                    check=False, timeout=30 * 60)
        record = {"label": label, "args": list(map(str, args)), "cwd": str(cwd or self.root),
                  "exitCode": result.returncode, "elapsedSeconds": round(time.monotonic() - started, 3)}
        self.commands.append(record)
        (self.output / (label + ".command.json")).write_text(json.dumps(record, indent=2) + "\n")
        if result.returncode != status:
            print((self.output / (label + ".stderr")).read_text(errors="replace")[-12000:], flush=True)
            raise RuntimeError(f"{label}: expected {status}, got {result.returncode}")
        return (self.output / (label + ".stdout")).read_text()

    def consumer(self, name, manifest, source):
        directory = self.output / name
        (directory / "src").mkdir(parents=True)
        (directory / "Cargo.toml").write_text(isolated_manifest(manifest, name))
        (directory / "src/lib.rs").write_bytes(source)
        shutil.copyfile(self.root / "Cargo.lock", directory / "Cargo.lock")
        return directory

    def metadata(self, label, directory, features=(), locked=False):
        flags = ["--locked"] if locked else []
        return json.loads(self.command(label, ["cargo", "metadata", "--offline", "--format-version", "1", *flags, *features], directory))

    def clean_consumer(self, label, directory):
        name = tomllib.loads((directory / "Cargo.toml").read_text())["package"]["name"]
        # Clear only this tiny generated root, preserving every provider/dependency.
        # This also makes fresh:false evidence reliable on sticky-worker reruns.
        self.command(label + "-clean-root", ["cargo", "clean", "--locked", "--offline", "--package", name, *PROFILE], directory)

    def build_once(self, label, mode, directory, features=()):
        # Match the native worker's ci profile so dependencies reuse its cache.
        self.clean_consumer(label + "-" + mode, directory)
        output = self.command(label + "-" + mode + "-build", ["cargo", "build", "--locked", "--offline", "--lib", "--message-format=json", *PROFILE, *features], directory)
        self.consumer_builds.append({"provider": label, "mode": mode, "artifact": verify_consumer_build(output, directory)})

    def builds(self, label, directory):
        for mode, features in [("default", []), ("distributed", ["--features", "distributed"])]:
            self.build_once(label, mode, directory, features)
            self.command(label + "-" + mode + "-test", ["cargo", "test", "--locked", "--offline", "--lib", *PROFILE, *features], directory)

    def run(self):
        require(self.command("source-head", ["git", "rev-parse", "HEAD"]).strip() == self.source_sha, "SOURCE_SHA differs from checkout")
        require(not self.command("source-status", ["git", "status", "--porcelain", "--untracked-files=no"]), "tracked source must be clean")
        self.command("cargo-version", ["cargo", "-Vv"])
        self.command("rustc-version", ["rustc", "-Vv"])
        root_manifest = tomllib.loads((self.root / "Cargo.toml").read_text())
        version = root_manifest["workspace"]["package"]["version"]
        providers = {}
        for alias in ALIASES:
            dependency = root_manifest["workspace"]["dependencies"][alias]
            require(dependency["package"] == "vize_" + alias and dependency["version"] == "=" + version,
                    "workspace alias/version mismatch")
            providers["vize_" + alias] = (self.root / dependency["path"] / "Cargo.toml", None)

        fixture = self.root / "tests/_fixtures/ci/checked-corsa-consumer"
        template = (fixture / "Cargo.toml").read_text()
        require(template.count('version = "*"') == len(OWNED), "fixture must define all six version slots")
        template = template.replace('version = "*"', 'version = "=' + version + '"')
        source = (fixture / "src/lib.rs").read_bytes()
        require("patch" not in tomllib.loads(template), "independent fixture must not patch dependencies")
        seed = template
        for alias in ALIASES:
            before = f'{alias} = {{ package = "vize_{alias}", version = "={version}" }}'
            require(seed.count(before) == 1, "fixture dependency is not an exact owned alias")
            seed = seed.replace(before, before[:-2] + ', path = ' + json.dumps(str(providers["vize_" + alias][0].parent)) + ' }')
        path_consumer = self.consumer("path-consumer", seed, source)
        graphs = []
        for mode, features in [("default", []), ("distributed", ["--features", "distributed"])]:
            graphs.append({"provider": "path", "mode": mode, "packages": verify_graph(
                self.metadata("path-" + mode + "-metadata", path_consumer, features), version, providers, bool(features))})
        self.builds("path", path_consumer)

        vendor = self.output / "archive-directory-source"
        self.command("vendor-registry-dependencies", ["cargo", "vendor", "--locked", "--offline", "--versioned-dirs", vendor], path_consumer)
        package_target = self.output / "package-target"
        self.command("generate-cargo-archives", ["cargo", "package", "--locked", "--offline", "--no-verify", "--exclude-lockfile",
            "--target-dir", package_target, *[arg for name in ALIASES for arg in ["-p", "vize_" + name]]])
        archives = []
        archive_providers = {}
        for alias in ALIASES:
            name, prefix = "vize_" + alias, "vize_" + alias + "-" + version
            archive = package_target / "package" / (prefix + ".crate")
            bodies = archive_members(archive, prefix)
            manifest = tomllib.loads(bodies["Cargo.toml"].decode())
            original = providers[name][0]
            source_members = verify_packaged_source(original.parent, bodies)
            require(bodies["Cargo.toml.orig"] == original.read_bytes(), "archive original manifest differs from source")
            require(manifest["package"]["name"] == name and manifest["package"]["version"] == version
                    and manifest["lib"]["name"] == alias, "archive package/lib identity mismatch")
            vcs = json.loads(bodies[".cargo_vcs_info.json"])
            require(vcs["git"]["sha1"] == self.source_sha and not vcs["git"].get("dirty", False), "archive source identity mismatch")
            require(vcs["path_in_vcs"] == str(original.parent.relative_to(self.root)), "archive source directory mismatch")
            require(manifest["package"].get("license") == tomllib.loads(original.read_text())["package"]["license"], "package license changed")
            for dependency in manifest.get("dependencies", {}).values():
                require("path" not in dependency, "archive retained a source path dependency")
                if dependency.get("package") in OWNED:
                    require(dependency["version"] == "=" + version, "archive internal version mismatch")
            destination = vendor / prefix
            destination.mkdir(exist_ok=False)
            for relative, body in bodies.items():
                target = destination / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(body)
            checksum = {"package": digest(archive.read_bytes()), "files": {relative: digest(body) for relative, body in bodies.items()}}
            (destination / ".cargo-checksum.json").write_text(json.dumps(checksum) + "\n")
            archives.append({"name": name, "version": version, "sha256": checksum["package"],
                             "sourceMembers": source_members, "members": checksum["files"]})
            archive_providers[name] = (destination / "Cargo.toml", REGISTRY)

        config = '[source.crates-io]\nreplace-with = "archive-consumer"\n[source.archive-consumer]\ndirectory = ' + json.dumps(str(vendor)) + '\n'
        packaged = self.consumer("archive-consumer", template, source)
        negative = self.consumer("default-feature-negative", template, b"pub use corsa::orchestrator::RaftConfig;\n")
        for directory in [packaged, negative]:
            (directory / ".cargo").mkdir()
            (directory / ".cargo/config.toml").write_text(config)
        for mode, features in [("default", []), ("distributed", ["--features", "distributed"])]:
            graphs.append({"provider": "normalized-archive", "mode": mode, "packages": verify_graph(
                self.metadata("archive-" + mode + "-metadata", packaged, features), version, archive_providers, bool(features))})
        self.builds("archive", packaged)
        verify_graph(self.metadata("negative-default-metadata", negative), version, archive_providers, False)
        self.clean_consumer("negative-default", negative)
        diagnostics = self.command("default-experimental-import-refused", ["cargo", "build", "--locked", "--offline", "--lib", "--message-format=json", *PROFILE], negative, 101)
        errors = [row["message"] for line in diagnostics.splitlines() if (row := json.loads(line)).get("reason") == "compiler-message" and row["message"]["level"] == "error"]
        require(len(errors) == 1 and errors[0].get("code", {}).get("code") == "E0432"
                and "corsa::orchestrator::RaftConfig" in errors[0]["message"], "default build failed for a reason other than gated RaftConfig")
        require(any(span["is_primary"] and (negative / span["file_name"]).resolve() == (negative / "src/lib.rs").resolve()
                    for span in errors[0]["spans"]), "negative diagnostic must originate in the independent refusal consumer")
        source_consumers = qualify_source_consumers(self, version, providers, verify_graph)
        require(not self.command("final-source-status", ["git", "status", "--porcelain", "--untracked-files=no"]), "acceptance mutated tracked source")
        require(self.command("final-source-head", ["git", "rev-parse", "HEAD"]).strip() == self.source_sha, "source identity changed")
        receipt = {"sourceSha": self.source_sha, "version": version, "fixtureSha256": digest(source), "commands": self.commands,
                   "graphs": graphs, "cliMetadata": source_consumers["cli"], "canonDefaultNative": source_consumers["canon"],
                   "archives": archives, "consumerBuilds": self.consumer_builds,
                   "pathDefaultAndDistributedBuilt": True,
                   "archiveDefaultAndDistributedBuilt": True, "consumerTestsPassed": True, "defaultExperimentalImportRefused": True,
                   "consumerPatches": False, "publicRegistryAcceptance": False, "cliVariantsCompiled": True,
                   "canonDefaultNativeCompiled": True,
                   "minimalStoryCollectionTestPassed": source_consumers["minimalStoryCollectionTestPassed"]}
        (self.output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, default=Path(os.environ.get("RUNNER_TEMP", "/tmp")) / "checked-corsa-package-consumer")
    parser.add_argument("--source-sha", default=os.environ.get("SOURCE_SHA"), required=not os.environ.get("SOURCE_SHA"))
    arguments = parser.parse_args()
    Acceptance(arguments.root.resolve(), arguments.output.resolve(), arguments.source_sha).run()
