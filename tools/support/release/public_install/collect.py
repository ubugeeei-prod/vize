"""Prospective public-install custody producer. No source executable fallback."""

import argparse
import datetime
import hashlib
import importlib.machinery
import importlib.util
import json
import os
import pathlib
import re
import subprocess
import sys

# -I ignores PYTHONPATH. Compile owned sibling sources directly: timestamp/size
# matching __pycache__ bytecode must never execute while we hash .py authority.
LOADED_SOURCES = {}


class SourceOnlyLoader(importlib.machinery.SourceFileLoader):
    def get_code(self, fullname):
        source = self.get_data(self.path)
        LOADED_SOURCES[pathlib.Path(self.path).name] = hashlib.sha256(source).hexdigest()
        return self.source_to_code(source, self.path)


for module_name in ("identity", "registry", "archive", "probe"):
    module_path = pathlib.Path(__file__).resolve().parent / (module_name + ".py")
    if module_path.resolve(strict=True) != module_path:
        raise ValueError("owned collector module cannot redirect through a symlink")
    loader = SourceOnlyLoader(module_name, str(module_path))
    specification = importlib.util.spec_from_loader(module_name, loader)
    module = importlib.util.module_from_spec(specification)
    sys.modules[module_name] = module
    loader.exec_module(module)
from identity import (OVERRIDES, digest, exact_path, fresh_outputs, producer_authority,
                      reject_overrides, source_file, source_identity, source_manifests)
from archive import archive_comparison, recheck_files
from registry import package_metadata, read_bytes
from probe import public_probe

PLATFORM = "@vizejs/native-darwin-arm64"
CORSA = "@typescript/typescript-darwin-arm64"
VIZE_PACKAGES = ("vize", "oxlint-plugin-vize", "@vizejs/native", PLATFORM)


def read_json(path):
    return json.loads(path.read_bytes())


def write_new(path, packet):
    with path.open("x", encoding="utf-8") as output:
        output.write(json.dumps(packet, indent=2) + "\n")


def collect(args):
    reject_overrides(os.environ)
    producer_root = pathlib.Path(__file__).resolve().parents[4]
    authority = producer_authority(producer_root, args.collector_sha256)
    for item in authority["files"]:
        name = pathlib.Path(item["path"]).name
        if name in LOADED_SOURCES and LOADED_SOURCES[name] != item["sha256"]:
            raise ValueError("executed collector source differs from reviewed source bytes")
    root = exact_path(args.root, directory=True)
    source = source_identity(root, args.cut, args.head, args.tag, args.run)
    source_declarations = source_manifests(root, source)
    version = args.tag[1:]
    install = exact_path(args.install_root, directory=True)
    if install == root or root in install.parents or (install / ".git").exists():
        raise ValueError("fresh public installation must be outside the source workspace")
    output, payload_manifest, journal = fresh_outputs(args.output)
    if install in output.parents:
        raise ValueError("custody outputs must be outside installed payloads")
    node = exact_path(args.node)
    if not os.access(node, os.X_OK):
        raise ValueError("explicit Node executable required")
    hook = exact_path(str(producer_root / "tools/support/release/public_install/native-custody.cjs"))
    lock_path = exact_path(str(install / "package-lock.json"), owner=install)
    lock_bytes = lock_path.read_bytes()
    lock = json.loads(lock_bytes)
    dependencies = {name: version for name in VIZE_PACKAGES}
    consumer = read_json(exact_path(str(install / "package.json"), owner=install))
    if (lock.get("lockfileVersion") != 3 or consumer.get("private") is not True
            or consumer.get("dependencies") != dependencies
            or lock.get("packages", {}).get("", {}).get("dependencies") != dependencies):
        raise ValueError("fresh private npm consumer and lock v3 with all four exact Vize versions required")
    cli_manifest = read_json(exact_path(str(install / "node_modules/vize/package.json"), owner=install))
    corsa_version = cli_manifest.get("optionalDependencies", {}).get(CORSA)
    if (not isinstance(corsa_version, str)
            or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?", corsa_version)
            or int(corsa_version.split(".")[0]) < 7):
        raise ValueError("exact public bundled Corsa version declaration missing")
    if (corsa_version != source_declarations["corsaVersion"]
            or cli_manifest.get("dependencies", {}).get("@vizejs/native") != version):
        raise ValueError("public CLI native/Corsa dependency differs from its actual H declaration")
    records, archive_receipts = [], []
    bundled_corsa = None
    plans = [(name, version, source) for name in VIZE_PACKAGES] + [(CORSA, corsa_version, None)]
    for name, expected, vize_source in plans:
        package_dir = exact_path(str(install / "node_modules" / name), owner=install, directory=True)
        manifest = read_json(exact_path(str(package_dir / "package.json"), owner=package_dir))
        entry = lock.get("packages", {}).get("node_modules/" + name, {})
        if (manifest.get("name") != name or manifest.get("version") != expected
                or entry.get("version") != expected or entry.get("link") is True):
            raise ValueError("installed package identity differs: " + name)
        proof = package_metadata(name, expected, entry, vize_source)
        comparison = archive_comparison(package_dir, read_bytes(entry["resolved"]), entry["integrity"])
        receipt = {"name": name, "version": expected, "resolved": entry["resolved"],
                   "integrity": entry["integrity"], "packageDirectory": str(package_dir),
                   "source": {"H": source["H"], "R": source["R"]} if vize_source else {"thirdPartyRegistry": True},
                   "provenancePayloadBinding": proof, **comparison}
        archive_receipts.append(receipt)
        if vize_source:
            records.append({"name": name, "version": version, "resolved": entry["resolved"],
                            "integrity": entry["integrity"], "provenanceSourceH": source["H"],
                            "provenanceR": source["R"], "provenancePayloadBinding": proof})
        else:
            executable = exact_path(str(package_dir / "lib/tsc"), owner=package_dir)
            if not os.access(executable, os.X_OK):
                raise ValueError("public bundled Corsa launcher is not executable")
            bundled_corsa = {"packageName": name, "version": expected, "declaredVersion": corsa_version,
                             "path": str(executable), "sha256": digest(executable.read_bytes()),
                             "resolved": entry["resolved"], "integrity": entry["integrity"],
                             "allInstalledFileBytesEqualPublicArchive": True}
    cli_bin = exact_path(str(install / "node_modules/vize" / cli_manifest["bin"]["vize"]),
                         owner=install / "node_modules/vize")
    dist_cli = exact_path(str(install / "node_modules/vize/dist/cli.mjs"), owner=install)
    if cli_bin.read_bytes() != source_file(root, source["H"], "npm/cli/bin/vize"):
        raise ValueError("public CLI wrapper differs from H")
    recheck_files(archive_receipts)
    write_new(payload_manifest, {"schema": "vize-public-registry-payload-files-v1", "installRoot": str(install),
                                "source": source, "packages": archive_receipts})
    payload_bytes = exact_path(str(payload_manifest)).read_bytes()
    probe = public_probe(node, hook, install, PLATFORM, cli_bin, journal, version,
                         pathlib.Path(bundled_corsa["path"]))
    recheck_files(archive_receipts)
    if (lock_path.read_bytes() != lock_bytes or exact_path(str(payload_manifest)).read_bytes() != payload_bytes
            or producer_authority(producer_root) != authority):
        raise ValueError("lock, payload manifest or reviewed collector bytes changed during collection")
    report = {"schema": "vize-public-registry-install-v1", "checkedAt": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "version": version, "source": source, "installRoot": str(install),
              "packageLockPath": str(lock_path), "packageLockSha256": digest(lock_bytes),
              "registry": records, "bundledCorsa": bundled_corsa,
              "publicArchiveComparison": [{key: value for key, value in item.items() if key != "files"} for item in archive_receipts],
              "payloadManifestPath": str(payload_manifest), "payloadManifestSha256": digest(payload_bytes),
              "node": probe["node"], "cli": {"binPath": str(cli_bin), "binSha256": digest(cli_bin.read_bytes()),
                "distCliPath": str(dist_cli), "distCliSha256": digest(dist_cli.read_bytes()),
                "versionStdout": probe["stdout"], "versionStderr": probe["stderr"], "versionExitCode": probe["exitCode"]},
              "native": probe["native"], "custodyHook": {"path": str(hook), "sha256": digest(hook.read_bytes()),
                "configurationEnvironment": "VIZE_PUBLIC_NATIVE_CUSTODY"},
              "sourceOverrides": {key: "" for key in OVERRIDES}, "collectorAuthority": authority,
              "sourceManifestPayloads": source_declarations["files"], "success": True}
    write_new(output, report)
    return {"output": str(output), "receiptSHA256": digest(exact_path(str(output)).read_bytes()), "version": version,
            "source": source["H"], "successfulReturnObserved": True, "success": True}


def main():
    reject_overrides(os.environ)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--print-collector-authority", action="store_true")
    for key in ("root", "cut", "head", "tag", "run", "install-root", "node", "output", "collector-sha256"):
        parser.add_argument("--" + key)
    args = parser.parse_args()
    if args.print_collector_authority:
        print(json.dumps(producer_authority(pathlib.Path(__file__).resolve().parents[4]), indent=2))
        return
    if any(getattr(args, key.replace("-", "_")) is None for key in
           ("root", "cut", "head", "tag", "run", "install-root", "node", "output", "collector-sha256")):
        parser.error("all explicit source/install/Node/output/reviewed collector arguments are required")
    print(json.dumps(collect(args)))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        sys.exit(str(error))
