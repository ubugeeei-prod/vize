"""Explicit source, producer, path and initial-environment boundaries."""

import hashlib
import json
import os
import pathlib
import re
import subprocess

OVERRIDES = (
    "NODE_OPTIONS", "VIZE_PREFER_WORKSPACE_BINDING", "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI", "VIZE_ALLOW_NATIVE_VERSION_MISMATCH", "CORSA_PATH",
    "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE",
)
OBSERVER_OVERRIDES = ("VIZE_PUBLIC_NATIVE_CUSTODY", "VIZE_OXLINT_NATIVE_CUSTODY")
PRODUCER_FILES = (
    "tools/commands/release/npm/collect-public-install.rs",
    *["tools/support/release/public_install/" + name for name in (
        "collect.py", "identity.py", "registry.py", "archive.py", "probe.py",
        "native-custody.cjs", "authority-schema.ts",
    )],
)


def reject_overrides(environment):
    nonempty = [key for key in OVERRIDES + OBSERVER_OVERRIDES if environment.get(key)]
    if nonempty:
        raise ValueError("initial source/runtime overrides must be empty: " + ", ".join(nonempty))


def digest(data):
    return hashlib.sha256(data).hexdigest()


def exact_path(value, owner=None, directory=False):
    path = pathlib.Path(value)
    if not path.is_absolute() or path.resolve(strict=True) != path:
        raise ValueError("absolute canonical paths without symlinks required: " + str(path))
    if owner is not None and owner not in path.parents:
        raise ValueError("path escaped its owner: " + str(path))
    if (directory and not path.is_dir()) or (not directory and not path.is_file()):
        raise ValueError("unexpected path type: " + str(path))
    return path


def fresh_outputs(value):
    output = pathlib.Path(value)
    if not output.is_absolute() or output.parent.resolve(strict=True) != output.parent:
        raise ValueError("absolute canonical output parent required")
    paths = (output, pathlib.Path(str(output) + ".public-payload-files.json"),
             pathlib.Path(str(output) + ".native-journal.jsonl"))
    if any(os.path.lexists(path) for path in paths):
        raise ValueError("receipt, payload manifest and native journal must all be new")
    return paths


def source_identity(root, cut, head, tag, run):
    if (not re.fullmatch(r"[0-9a-f]{40}", cut)
            or not re.fullmatch(r"[0-9a-f]{40}", head)
            or not re.fullmatch(r"v0\.[0-9]+\.0", tag)
            or not re.fullmatch(r"[1-9][0-9]*", run)):
        raise ValueError("explicit full C/H/stable minor tag/Release run required")
    for commit in (cut, head):
        kind = subprocess.check_output(
            ["git", "--no-replace-objects", "cat-file", "-t", commit], cwd=root, text=True).strip()
        if kind != "commit":
            raise ValueError("immutable source object must be a commit")
    raw = subprocess.check_output(["git", "--no-replace-objects", "cat-file", "commit", head], cwd=root)
    # rev-list can still obey legacy grafts even with replacements disabled.
    parents = [line[7:].decode() for line in raw.split(b"\n\n", 1)[0].splitlines()
               if line.startswith(b"parent ")]
    if parents != [cut]:
        raise ValueError("H must be the exact single-parent immutable C cut")
    return {"C": cut, "H": head, "tag": tag, "R": run}


def source_file(root, head, path):
    return subprocess.check_output(["git", "--no-replace-objects", "cat-file", "blob", head + ":" + path], cwd=root)


def catalog_version(workspace, catalog, package):
    catalogs = re.findall(r"^  " + re.escape(catalog) + r":\n((?:    [^\n]*\n|\n)*)", workspace.decode(), re.M)
    entries = re.findall(r'^    "' + re.escape(package) + r'": "([^"]+)"$',
                         catalogs[0], re.M) if len(catalogs) == 1 else []
    if len(entries) != 1:
        raise ValueError("H exact named catalog declaration missing or duplicated")
    return entries[0]


def source_manifests(root, source):
    version = source["tag"][1:]
    manifests = (
        ("npm/cli/package.json", "vize"),
        ("npm/oxlint/package.json", "oxlint-plugin-vize"),
        ("npm/native/package.json", "@vizejs/native"),
    )
    receipts = []
    workspace = source_file(root, source["H"], "pnpm-workspace.yaml")
    if catalog_version(workspace, "native-binaries", "@vizejs/native-darwin-arm64") != version:
        raise ValueError("H platform package catalog version differs from requested version")
    corsa = catalog_version(workspace, "corsa-runtime", "@typescript/typescript-darwin-arm64")
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?", corsa) or int(corsa.split(".")[0]) < 7:
        raise ValueError("H exact public Corsa catalog version required")
    receipts.append({"path": "pnpm-workspace.yaml", "sha256": digest(workspace)})
    for path, expected_name in manifests:
        data = source_file(root, source["H"], path)
        manifest = json.loads(data)
        if (manifest.get("name") != expected_name or manifest.get("version") != version
                or manifest.get("private") is True):
            raise ValueError("H source manifest differs from requested public version: " + path)
        if expected_name == "vize":
            if manifest.get("dependencies", {}).get("@vizejs/native") not in ("workspace:*", version):
                raise ValueError("H CLI native workspace edge differs from requested native version")
            if manifest.get("optionalDependencies", {}).get("@typescript/typescript-darwin-arm64") not in ("catalog:corsa-runtime", corsa):
                raise ValueError("H CLI bundled Corsa declaration differs from its catalog")
        else:
            declaration = manifest.get("optionalDependencies", {}).get("@vizejs/native-darwin-arm64")
            if declaration not in (version, "catalog:native-binaries"):
                raise ValueError("H native platform declaration differs from requested version")
        if expected_name == "@vizejs/native" and "aarch64-apple-darwin" not in manifest.get("napi", {}).get("targets", []):
            raise ValueError("H does not declare the Darwin ARM64 native target")
        receipts.append({"path": path, "sha256": digest(data)})
    return {"files": receipts, "corsaVersion": corsa}


def producer_authority(root, expected=None):
    files = [{"path": name, "sha256": digest(exact_path(str(root / name)).read_bytes())}
             for name in sorted(PRODUCER_FILES)]
    snapshot = digest("".join(item["path"] + "\0" + item["sha256"] + "\n"
                              for item in files).encode())
    if expected is not None and (not re.fullmatch(r"[0-9a-f]{64}", expected)
                                 or snapshot != expected):
        raise ValueError("reviewed collector snapshot digest differs")
    return {"schema": "vize-public-install-collector-v1", "sha256": snapshot, "files": files,
            "scope": "reviewed collector tool snapshot; not a claim these tools are in H"}
