#!/usr/bin/env python3
"""Small refusal tests; real builds run only in checked-corsa-package-consumer.py."""

import copy
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

import checked_corsa_source_consumers as source_consumers

SPEC = importlib.util.spec_from_file_location("consumer", Path(__file__).with_name("checked-corsa-package-consumer.py"))
consumer = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(consumer)


class ArchiveSafety(unittest.TestCase):
    def archive(self, members):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        archive = Path(directory.name) / "package.crate"
        with tarfile.open(archive, "w:gz") as stream:
            for name, kind in members:
                member = tarfile.TarInfo(name)
                member.type = kind
                member.size = 4 if kind == tarfile.REGTYPE else 0
                stream.addfile(member, io.BytesIO(b"body") if member.size else None)
        return archive

    def test_regular_package_members_are_read(self):
        archive = self.archive([("crate-1/src/lib.rs", tarfile.REGTYPE)])
        self.assertEqual(consumer.archive_members(archive, "crate-1"), {"src/lib.rs": b"body"})

    def test_tar_paths_and_links_cannot_escape_or_replace_files(self):
        for names in [[("crate-1/../escape", tarfile.REGTYPE)], [("foreign/src/lib.rs", tarfile.REGTYPE)],
                      [("crate-1/src/link", tarfile.SYMTYPE)], [("crate-1/src/link", tarfile.LNKTYPE)],
                      [("crate-1/src/lib.rs", tarfile.REGTYPE)] * 2]:
            with self.subTest(names=names), self.assertRaises(ValueError):
                consumer.archive_members(self.archive(names), "crate-1")


class GraphIsolation(unittest.TestCase):
    def setUp(self):
        self.providers = {name: (Path("/consumer") / name / "Cargo.toml", None) for name in consumer.OWNED}
        self.metadata = {"packages": [{"id": name, "name": name, "version": "0.999.0", "source": None,
                                      "manifest_path": str(self.providers[name][0])} for name in sorted(consumer.OWNED)],
                         "resolve": {"nodes": [{"id": name, "features": []} for name in sorted(consumer.OWNED)]}}
        self.metadata["packages"].append({"id": "runtime", "name": "corsa_runtime", "version": consumer.RUNTIME_VERSION,
                                          "source": consumer.REGISTRY})

    def check(self, metadata, distributed=False):
        return consumer.verify_graph(metadata, "0.999.0", self.providers, distributed)

    def test_shared_graph_and_requested_distributed_feature(self):
        self.assertEqual(len(self.check(self.metadata)), 6)
        for node in self.metadata["resolve"]["nodes"]:
            if node["id"] in {"vize_corsa", "vize_corsa_orchestrator"}:
                node["features"] = ["experimental-distributed"]
        self.assertEqual(len(self.check(self.metadata, True)), 6)
        with self.assertRaisesRegex(ValueError, "experimental feature"):
            self.check(self.metadata)

    def test_rejects_upstream_duplicate_wrong_path_version_and_runtime_downgrade(self):
        mutations = [lambda data: data["packages"].append({"name": "corsa_core"}),
                     lambda data: data["packages"].append(copy.deepcopy(data["packages"][0])),
                     lambda data: data["packages"][0].update(manifest_path="/unexpected/Cargo.toml"),
                     lambda data: data["packages"][0].update(version="0.998.0"),
                     lambda data: data["packages"][-1].update(version="1.14.0"),
                     lambda data: data["packages"][-1].update(source=None)]
        for mutation in mutations:
            data = copy.deepcopy(self.metadata)
            mutation(data)
            with self.subTest(data=data), self.assertRaises(ValueError):
                self.check(data)


class PackageInventory(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        source = {"Cargo.toml": b'[package]\nname="package"\nexclude=["Cargo.toml.orig", ".cargo_vcs_info.json"]\n',
                  "Cargo.toml.orig": b"upstream manifest", ".cargo_vcs_info.json": b"upstream receipt",
                  "Cargo.lock": b"source lock", "LICENSE": b"MIT license text\n", "README.md": b"docs\n",
                  "src/lib.rs": b"pub fn api() {}\n", "tests/fixture.txt": b"retained fixture\n"}
        for relative, body in source.items():
            path = self.directory / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(body)
        self.bodies = {name: body for name, body in source.items() if name not in consumer.GENERATED}
        self.bodies.update({"Cargo.toml": b"normalized manifest", "Cargo.toml.orig": source["Cargo.toml"],
                            ".cargo_vcs_info.json": b"current receipt"})

    def test_full_source_inventory_allows_only_regenerated_reserved_files(self):
        self.assertEqual(consumer.verify_packaged_source(self.directory, self.bodies),
                         ["LICENSE", "README.md", "src/lib.rs", "tests/fixture.txt"])

    def test_omitted_source_test_or_extra_member_is_refused(self):
        for missing in ["src/lib.rs", "tests/fixture.txt", "README.md", "Cargo.toml.orig"]:
            bodies = self.bodies.copy()
            del bodies[missing]
            with self.subTest(missing=missing), self.assertRaisesRegex(ValueError, "inventory differs"):
                consumer.verify_packaged_source(self.directory, bodies)
        with self.assertRaisesRegex(ValueError, "inventory differs"):
            consumer.verify_packaged_source(self.directory, {**self.bodies, "Cargo.lock": b"unexpected lock"})

    def test_missing_or_changed_license_is_refused_explicitly(self):
        missing = self.bodies.copy()
        del missing["LICENSE"]
        with self.assertRaisesRegex(ValueError, "preserve LICENSE"):
            consumer.verify_packaged_source(self.directory, missing)
        with self.assertRaisesRegex(ValueError, "LICENSE bytes changed"):
            consumer.verify_packaged_source(self.directory, {**self.bodies, "LICENSE": b"different license"})

    def test_manifest_cannot_hide_source_exclusions(self):
        (self.directory / "Cargo.toml").write_text('[package]\nexclude=["src/lib.rs"]\n')
        with self.assertRaisesRegex(ValueError, "reserved Cargo"):
            consumer.verify_packaged_source(self.directory, self.bodies)


class ConsumerCacheIsolation(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        self.manifest = '[package]\nname = "checked-corsa-consumer"\nversion = "0.0.0"\n[workspace]\n'
        (self.directory / "Cargo.toml").write_text(consumer.isolated_manifest(self.manifest, "archive-consumer"))
        self.artifact = {"reason": "compiler-artifact", "manifest_path": str(self.directory / "Cargo.toml"),
                         "package_id": "generated-root", "fresh": False, "features": [], "filenames": ["consumer.rlib"],
                         "target": {"kind": ["lib"], "name": "checked_corsa_consumer_archive_consumer",
                                    "src_path": str(self.directory / "src/lib.rs")}}

    def test_sibling_consumers_cannot_share_package_identity(self):
        names = [consumer.tomllib.loads(consumer.isolated_manifest(self.manifest, name))["package"]["name"]
                 for name in ["path-consumer", "archive-consumer", "default-feature-negative"]]
        self.assertEqual(len(set(names)), 3)

    def test_build_requires_compilation_of_the_correct_root_source(self):
        self.assertFalse(consumer.verify_consumer_build(json.dumps(self.artifact), self.directory)["fresh"])
        for fields in [{"fresh": True}, {"target": {**self.artifact["target"], "src_path": "/sibling/src/lib.rs"}},
                       {"manifest_path": "/sibling/Cargo.toml"}]:
            with self.subTest(fields=fields), self.assertRaises(ValueError):
                consumer.verify_consumer_build(json.dumps({**self.artifact, **fields}), self.directory)

    def test_each_mode_clears_only_its_generated_root_before_compiling(self):
        runner = consumer.Acceptance.__new__(consumer.Acceptance)
        runner.consumer_builds = []
        commands = []

        def command(label, args, directory):
            commands.append(args)
            return json.dumps(self.artifact) if args[1] == "build" else ""

        runner.command = command
        runner.builds("archive", self.directory)
        self.assertEqual([args[1] for args in commands], ["clean", "build", "test", "clean", "build", "test"])
        for args in commands[::3]:
            self.assertEqual(args[args.index("--package") + 1], "checked-corsa-consumer-archive-consumer")
            self.assertNotIn("--workspace", args)
        self.assertEqual(len(runner.consumer_builds), 2)


class SourceConsumerContracts(unittest.TestCase):
    def test_default_canon_is_not_artificially_enabled_or_patched(self):
        manifest = consumer.tomllib.loads(source_consumers.source_manifest("vize_canon", Path("/source/canon")))
        self.assertEqual(manifest["dependencies"], {"vize_canon": {"path": "/source/canon"}})
        self.assertNotIn("patch", manifest)

    def test_all_six_external_sources_must_reach_actual_library_builds(self):
        root = Path(__file__).resolve().parents[4]

        class Runner:
            def __init__(self):
                self.root, self.records, self.builds = root, {}, []

            def consumer(self, name, manifest, source):
                directory = Path("/external") / name
                self.records[directory] = (consumer.tomllib.loads(manifest), source)
                return directory

            def metadata(self, label, directory):
                dependencies = self.records[directory][0]["dependencies"]
                requested = dependencies.get("vize", {})
                features = set(requested.get("features", []))
                if requested.get("default-features", True):
                    features.add("default")
                if "default" in features:
                    features.update(["glyph", "maestro"])
                packages = [{"id": name, "name": name, "source": None,
                             "manifest_path": str(root / "crates" / name / "Cargo.toml")}
                            for name in ["vize", "vize_canon"]]
                return {"packages": packages, "resolve": {"nodes": [
                    {"id": "vize", "features": sorted(features)}, {"id": "vize_canon", "features": ["default", "native"]}]}}

            def build_once(self, provider, mode, directory):
                self.builds.append((provider, mode, directory))

            def command(self, label, args):
                self.test_command = args
                return f"test {source_consumers.MINIMAL_STORY_TEST} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n"

        runner = Runner()
        result = source_consumers.qualify_source_consumers(runner, "0.999.0", {}, lambda *args: [])
        self.assertEqual([(provider, mode) for provider, mode, _ in runner.builds],
                         [("canon-source", "default-native"), *[("cli-source", name) for name in ["default", "minimal", "glyph", "maestro", "all"]]])
        self.assertIn("--no-default-features", runner.test_command)
        self.assertIn(source_consumers.MINIMAL_STORY_TEST, runner.test_command)
        self.assertTrue(result["minimalStoryCollectionTestPassed"])
        for record, graph in zip(list(runner.records.values())[1:], result["cli"]):
            manifest, source = record
            self.assertEqual(set(manifest["dependencies"]), {"vize"})
            self.assertNotIn("patch", manifest)
            self.assertIn(b"vize::cli::run_from_args", source)
            self.assertEqual(b"vize::glyph::FormatOptions" in source, "glyph" in graph["features"])
            self.assertEqual(b"vize::maestro::serve_blocking" in source, "maestro" in graph["features"])

    def test_source_feature_metadata_rejects_registry_or_foreign_provider(self):
        metadata = {"packages": [{"id": "canon", "name": "vize_canon", "source": consumer.REGISTRY,
                                   "manifest_path": "/source/Cargo.toml"}], "resolve": {"nodes": []}}
        with self.assertRaisesRegex(ValueError, "unexpected vize_canon"):
            source_consumers.package_features(metadata, "vize_canon", Path("/source/Cargo.toml"))

    def test_missing_filtered_or_unrelated_minimal_story_execution_is_refused(self):
        for output in ["", "test result: ok. 0 passed; 0 failed;", "test unrelated ... ok\ntest result: ok. 1 passed; 0 failed;"]:
            with self.subTest(output=output), self.assertRaisesRegex(ValueError, "one exact regression"):
                source_consumers.verify_minimal_story_test(output)


if __name__ == "__main__":
    unittest.main()
