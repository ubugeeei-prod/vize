"""Registry-only graph and downloaded archive contracts; no network operations."""

import importlib.util
import json
from pathlib import Path
import re
import tomllib

SPEC = importlib.util.spec_from_file_location("checked_corsa_consumer", Path(__file__).with_name("checked-corsa-package-consumer.py"))
base = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(base)
PUBLIC = base.OWNED | {"vize_canon"}


def validate_identity(version, source_sha):
    base.require(re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", version), "explicit exact release version required")
    base.require(re.fullmatch(r"[0-9a-f]{40}", source_sha), "explicit 40-character release source SHA required")


def check_outside_workspace(output, root):
    output, root = output.resolve(), root.resolve()
    base.require(not output.is_relative_to(root), "public consumer output must be outside the source workspace")
    for ancestor in [output, *output.parents]:
        base.require(not any((ancestor / ".cargo" / name).exists() for name in ["config", "config.toml"]),
                     "ancestor Cargo configuration could replace public dependency sources")


def public_environment(environment, home, target):
    isolated = {key: value for key, value in environment.items()
                if not key.startswith("CARGO_") and not key.startswith(("RUSTC", "RUSTDOC", "RUSTFLAGS"))}
    isolated.update(CARGO_HOME=str(home), CARGO_TARGET_DIR=str(target), CARGO_TERM_COLOR="never", CARGO_INCREMENTAL="0")
    return isolated


def verify_public_graph(metadata, version, home, directory, distributed, canon=False):
    packages = metadata["packages"]
    root_id = metadata["resolve"]["root"]
    root = next(package for package in packages if package["id"] == root_id)
    base.require(root["source"] is None and Path(root["manifest_path"]).resolve() == (directory / "Cargo.toml").resolve(),
                 "unexpected public consumer root")
    registry = (home / "registry/src").resolve()
    for package in packages:
        if package["id"] == root_id:
            continue
        manifest = Path(package["manifest_path"]).resolve()
        base.require(package["source"] == base.REGISTRY and manifest.parent.parent.parent == registry
                     and manifest.name == "Cargo.toml" and manifest.parent.name == package["name"] + "-" + package["version"],
                     "public dependency must use the fresh crates.io source: " + package["name"])
    providers = {package["name"]: (Path(package["manifest_path"]), base.REGISTRY)
                 for package in packages if package["name"] in base.OWNED}
    owned = base.verify_graph(metadata, version, providers, distributed)
    if canon:
        candidates = [package for package in packages if package["name"] == "vize_canon"]
        base.require(len(candidates) == 1 and candidates[0]["version"] == version, "published Canon exact version required")
        features = next(node["features"] for node in metadata["resolve"]["nodes"] if node["id"] == candidates[0]["id"])
        base.require({"default", "native"} <= set(features), "published Canon must activate default native APIs")
    return {"ownedPackages": owned, "packages": [{key: package[key] for key in ["id", "name", "version", "source", "manifest_path"]}
                                                  for package in packages]}


def archive_evidence(package, home, lock, source_sha, source_directory):
    name, version = package["name"], package["version"]
    prefix = name + "-" + version
    archives = list((home / "registry/cache").glob("*/" + prefix + ".crate"))
    base.require(len(archives) == 1, "downloaded public archive missing or ambiguous: " + name)
    archive = archives[0]
    checksum = base.digest(archive.read_bytes())
    entries = [entry for entry in lock["package"] if entry["name"] == name and entry["version"] == version]
    base.require(len(entries) == 1 and entries[0]["source"] == base.REGISTRY and entries[0]["checksum"] == checksum,
                 "downloaded archive does not match the public registry lock checksum")
    bodies = base.archive_members(archive, prefix)
    manifest = tomllib.loads(bodies["Cargo.toml"].decode())
    base.require(manifest["package"]["name"] == name and manifest["package"]["version"] == version
                 and manifest["package"]["license"] == "MIT", "public archive manifest identity/license mismatch")
    vcs = json.loads(bodies[".cargo_vcs_info.json"])
    base.require(vcs["git"]["sha1"] == source_sha, "public archive source SHA mismatch")
    expected_path = "crates/vize_canon" if name == "vize_canon" else "vendor/" + name.removeprefix("vize_")
    base.require(vcs["path_in_vcs"] == expected_path, "public archive source directory mismatch")
    # Publisher staging strips dev-dependencies with --allow-dirty. Keep the
    # original VCS receipt, including truthful dirty:true, rather than rewriting it.
    base.require("LICENSE" in bodies and bodies["LICENSE"] == (source_directory / "LICENSE").read_bytes(),
                 "public archive must retain the complete expected MIT notice")
    expected = {path.relative_to(source_directory).as_posix() for path in source_directory.rglob("*") if path.is_file()} - base.GENERATED
    base.require(set(bodies) - base.GENERATED == expected, "public archive omitted or added source members")
    extracted = Path(package["manifest_path"]).parent
    actual = {path.relative_to(extracted).as_posix() for path in extracted.rglob("*") if path.is_file()} - {".cargo-ok", ".cargo-checksum.json"}
    base.require(actual == set(bodies), "extracted registry source differs from complete archive membership")
    for relative, body in bodies.items():
        base.require((extracted / relative).read_bytes() == body, "compiled registry source differs from archive: " + relative)
        if relative not in base.GENERATED:
            base.require((source_directory / relative).read_bytes() == body, "published source differs from release checkout: " + relative)
    return {"name": name, "version": version, "source": package["source"], "archive": str(archive), "checksum": checksum,
            "vcs": vcs, "normalizedManifest": manifest, "originalManifest": bodies["Cargo.toml.orig"].decode(),
            "members": {relative: base.digest(body) for relative, body in sorted(bodies.items())}}
