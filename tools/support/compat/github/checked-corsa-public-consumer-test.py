#!/usr/bin/env python3
"""Offline fixture controls only. Does not contact any package registry."""

import copy
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

import checked_corsa_public_contract as contract

SPEC = importlib.util.spec_from_file_location("public_consumer", Path(__file__).with_name("checked-corsa-public-consumer.py"))
public = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(public)
VERSION, SHA = "0.999.0", "b" * 40


class PublicGraph(unittest.TestCase):
    def setUp(self):
        self.home, self.directory = Path("/fresh/cargo-home"), Path("/fresh/consumer")
        self.metadata = {"packages": [], "resolve": {"root": "consumer", "nodes": []}}
        for name in sorted(contract.PUBLIC | {"corsa_runtime"}):
            version = contract.base.RUNTIME_VERSION if name == "corsa_runtime" else VERSION
            manifest = self.home / "registry/src/index.crates.io-fixture" / (name + "-" + version) / "Cargo.toml"
            self.metadata["packages"].append({"id": name, "name": name, "version": version,
                                               "source": contract.base.REGISTRY, "manifest_path": str(manifest)})
            self.metadata["resolve"]["nodes"].append({"id": name, "features": ["default", "native"] if name == "vize_canon" else []})
        self.metadata["packages"].append({"id": "consumer", "name": "consumer", "version": "0.0.0", "source": None,
                                           "manifest_path": str(self.directory / "Cargo.toml")})

    def check(self, metadata, distributed=False):
        return contract.verify_public_graph(metadata, VERSION, self.home, self.directory, distributed, canon=True)

    def test_real_registry_graph_and_canon_default_features(self):
        self.assertEqual(len(self.check(self.metadata)["ownedPackages"]), 6)
        for node in self.metadata["resolve"]["nodes"]:
            if node["id"] in {"vize_corsa", "vize_corsa_orchestrator"}:
                node["features"] = ["experimental-distributed"]
        self.check(self.metadata, True)
        with self.assertRaises(ValueError):
            self.check(self.metadata)

    def test_source_provider_version_runtime_and_native_feature_spoofs_are_refused(self):
        mutations = [lambda data: data["packages"][0].update(source=None),
                     lambda data: data["packages"][0].update(source="registry+https://foreign.invalid/index"),
                     lambda data: data["packages"][0].update(manifest_path="/source/vendor/Cargo.toml"),
                     lambda data: data["packages"][0].update(version="1.14.0"),
                     lambda data: next(node for node in data["resolve"]["nodes"] if node["id"] == "vize_canon").update(features=[]),
                     lambda data: data["packages"][-1].update(source=contract.base.REGISTRY),
                     lambda data: next(package for package in data["packages"] if package["name"] == "vize_corsa").update(version="0.998.0")]
        for mutation in mutations:
            data = copy.deepcopy(self.metadata)
            mutation(data)
            with self.subTest(data=data), self.assertRaises(ValueError):
                self.check(data)


class IndependentConfiguration(unittest.TestCase):
    def test_source_and_compiler_environment_overrides_are_removed(self):
        environment = {"PATH": "/bin", "CARGO_REGISTRIES_CRATES_IO_INDEX": "foreign", "CARGO_SOURCE_CRATES_IO_REPLACE_WITH": "local",
                       "CARGO_HOME": "/existing", "RUSTC_WRAPPER": "fake", "RUSTFLAGS": "--cfg fake", "CARGO_NET_OFFLINE": "true"}
        isolated = contract.public_environment(environment, Path("/fresh/home"), Path("/fresh/target"))
        self.assertEqual(isolated["CARGO_HOME"], "/fresh/home")
        self.assertEqual(isolated["CARGO_TARGET_DIR"], "/fresh/target")
        self.assertNotIn("CARGO_REGISTRIES_CRATES_IO_INDEX", isolated)
        self.assertNotIn("CARGO_SOURCE_CRATES_IO_REPLACE_WITH", isolated)
        self.assertNotIn("RUSTC_WRAPPER", isolated)
        self.assertNotIn("RUSTFLAGS", isolated)
        self.assertNotIn("CARGO_NET_OFFLINE", isolated)

    def test_workspace_ancestor_configuration_and_nonexact_versions_are_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            contract.check_outside_workspace(root / "outside", root / "source")
            with self.assertRaises(ValueError):
                contract.check_outside_workspace(root / "source/output", root / "source")
            (root / ".cargo").mkdir()
            (root / ".cargo/config.toml").write_text('[source.crates-io]\nreplace-with="local"\n')
            with self.assertRaises(ValueError):
                contract.check_outside_workspace(root / "outside", root / "source")
        for version in ["*", "^0.999", ">=0.999.0", "v0.999.0", "0.999.0\n"]:
            with self.subTest(version=version), self.assertRaises(ValueError):
                contract.validate_identity(version, SHA)
        for sha in ["HEAD", "b" * 39, "g" * 40]:
            with self.subTest(sha=sha), self.assertRaises(ValueError):
                contract.validate_identity(VERSION, sha)

    def test_consumer_paths_patches_and_version_ranges_are_refused(self):
        runner = public.PublicAcceptance.__new__(public.PublicAcceptance)
        runner.version = VERSION
        for dependency, suffix in [('version = "=' + VERSION + '", path = "/vendor"', ""),
                                   ('version = "=' + VERSION + '"', '\n[patch.crates-io]\n'), ('version = "*"', "")]:
            manifest = '[package]\nname = "consumer"\nversion = "0.0.0"\n[dependencies]\nvize_canon = { ' + dependency + ' }\n' + suffix
            with self.subTest(dependency=dependency), self.assertRaises(ValueError):
                runner.consumer("rejected", manifest, b"// never written")


class PublicArchive(unittest.TestCase):
    def fixture(self, changes=None, name="vize_corsa_core"):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        source, home = root / "release-source", root / "cargo-home"
        prefix = name + "-" + VERSION
        bodies = {"Cargo.toml": ('[package]\nname="' + name + '"\nversion="' + VERSION + '"\nlicense="MIT"\n').encode(),
                  "Cargo.toml.orig": b"[package]\nversion.workspace=true\n", "src/lib.rs": b"pub fn actual() {}\n",
                  "LICENSE": (Path(__file__).resolve().parents[4] / "LICENSE").read_bytes(), "tests/control.txt": b"retained\n",
                  ".cargo_vcs_info.json": json.dumps({"git": {"sha1": SHA, "dirty": True}, "path_in_vcs":
                      "crates/vize_canon" if name == "vize_canon" else "vendor/corsa_core"}).encode()}
        for relative, body in bodies.items():
            path = source / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(body)
        if changes:
            changes(bodies)
        archive = home / "registry/cache/index.crates.io-fixture" / (prefix + ".crate")
        archive.parent.mkdir(parents=True)
        extracted = home / "registry/src/index.crates.io-fixture" / prefix
        with tarfile.open(archive, "w:gz") as stream:
            for relative, body in bodies.items():
                member = tarfile.TarInfo(prefix + "/" + relative)
                member.size = len(body)
                stream.addfile(member, io.BytesIO(body))
                path = extracted / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(body)
        package = {"name": name, "version": VERSION, "source": contract.base.REGISTRY, "manifest_path": str(extracted / "Cargo.toml")}
        lock = {"package": [{"name": name, "version": VERSION, "source": contract.base.REGISTRY, "checksum": contract.base.digest(archive.read_bytes())}]}
        return package, home, lock, source

    def test_complete_archive_retains_checksum_source_and_truthful_dirty_receipt(self):
        package, home, lock, source = self.fixture()
        evidence = contract.archive_evidence(package, home, lock, SHA, source)
        self.assertTrue(evidence["vcs"]["git"]["dirty"])
        self.assertIn("tests/control.txt", evidence["members"])
        self.assertEqual(evidence["checksum"], lock["package"][0]["checksum"])

    def test_license_source_member_and_vcs_spoofs_are_refused(self):
        for mutation in [lambda bodies: bodies.pop("LICENSE"), lambda bodies: bodies.pop("tests/control.txt"),
                         lambda bodies: bodies.update({".cargo_vcs_info.json": json.dumps({"git": {"sha1": "c" * 40}, "path_in_vcs": "vendor/corsa_core"}).encode()})]:
            package, home, lock, source = self.fixture(mutation)
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                contract.archive_evidence(package, home, lock, SHA, source)

    def test_canon_requires_its_complete_release_mit_notice(self):
        package, home, lock, source = self.fixture(name="vize_canon")
        contract.archive_evidence(package, home, lock, SHA, source)
        package, home, lock, source = self.fixture(lambda bodies: bodies.update(LICENSE=b"MIT"), name="vize_canon")
        with self.assertRaisesRegex(ValueError, "complete expected MIT notice"):
            contract.archive_evidence(package, home, lock, SHA, source)

    def test_checksum_and_extracted_compiler_inputs_cannot_change(self):
        package, home, lock, source = self.fixture()
        lock["package"][0]["checksum"] = "0" * 64
        with self.assertRaises(ValueError):
            contract.archive_evidence(package, home, lock, SHA, source)
        lock["package"][0]["checksum"] = contract.base.digest(next((home / "registry/cache").glob("*/*.crate")).read_bytes())
        (Path(package["manifest_path"]).parent / "src/lib.rs").write_bytes(b"changed compiler input")
        with self.assertRaises(ValueError):
            contract.archive_evidence(package, home, lock, SHA, source)


if __name__ == "__main__":
    unittest.main()
