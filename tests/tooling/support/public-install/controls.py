"""Inert rejection controls. No public installation, native success or Corsa execution."""

import base64
import copy
import hashlib
import io
import json
import os
import pathlib
import py_compile
import runpy
import shutil
import subprocess
import sys
import tarfile
import tempfile
import types
import unittest
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[4]
sys.dont_write_bytecode = True
COLLECTOR = runpy.run_path(str(ROOT / "tools/support/release/public_install/collect.py"))
from archive import archive_comparison, recheck_files
from identity import OVERRIDES, PRODUCER_FILES, digest, exact_path, fresh_outputs, producer_authority, reject_overrides, source_identity, source_manifests
from registry import package_metadata, provenance_payload_binding, public_url, sri_sha512
from probe import verify_journal


class ArchiveControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.package = pathlib.Path(self.temporary.name).resolve() / "package"
        self.package.mkdir()
        self.files = {"package.json": b'{}', "dist/cli.mjs": b"inert JavaScript routing",
                      "native-targets.js": b"inert target map", "vize.node": b"not a native library",
                      "lib/tsc": b"inert Corsa launcher"}
        for name, data in self.files.items():
            path = self.package / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)

    def packet(self, entries=None):
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode="w:gz") as archive:
            for name, data in entries or [("package/" + key, value) for key, value in self.files.items()]:
                member = tarfile.TarInfo(name)
                member.size = len(data)
                archive.addfile(member, io.BytesIO(data))
        payload = stream.getvalue()
        integrity = "sha512-" + base64.b64encode(hashlib.sha512(payload).digest()).decode()
        return payload, integrity

    def rejected(self, entries=None):
        with self.assertRaises((ValueError, FileNotFoundError)):
            archive_comparison(self.package, *self.packet(entries))

    def test_entire_inert_file_set_is_compared(self):
        receipt = archive_comparison(self.package, *self.packet())
        self.assertEqual(receipt["fileCount"], 5)
        self.assertEqual({pathlib.Path(item["path"]).relative_to(self.package).as_posix()
                          for item in receipt["files"]}, set(self.files))

    def test_tampered_js_native_targets_native_and_corsa_are_rejected(self):
        for name in ("dist/cli.mjs", "native-targets.js", "vize.node", "lib/tsc"):
            with self.subTest(name=name):
                (self.package / name).write_bytes(b"tampered")
                self.rejected()
                (self.package / name).write_bytes(self.files[name])

    def test_missing_extra_wrong_duplicate_archive_files_are_rejected(self):
        entries = [("package/" + key, data) for key, data in self.files.items()]
        self.rejected(entries[:-1])
        self.rejected(entries + [("package/missing.js", b"extra")])
        self.rejected(entries + [entries[0]])
        self.rejected([("wrong/" + key, data) for key, data in self.files.items()])
        (self.package / "extra.js").write_bytes(b"extra installed file")
        self.rejected()

    def test_path_traversal_and_redirects_are_rejected(self):
        for name in ("package/../escape", "/package/package.json", "package/./package.json",
                     "package//package.json", "package\\package.json"):
            self.rejected([(name, b"inert")])
        target = self.package / "dist/cli.mjs"
        target.unlink()
        target.symlink_to(self.package / "native-targets.js")
        self.rejected()

    def test_tar_symlink_and_sri_mismatch_are_rejected(self):
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode="w:gz") as archive:
            member = tarfile.TarInfo("package/redirect")
            member.type, member.linkname = tarfile.SYMTYPE, "../outside"
            archive.addfile(member)
        payload = stream.getvalue()
        integrity = "sha512-" + base64.b64encode(hashlib.sha512(payload).digest()).decode()
        with self.assertRaises(ValueError):
            archive_comparison(self.package, payload, integrity)
        with self.assertRaises(ValueError):
            archive_comparison(self.package, self.packet()[0], "sha512-" + base64.b64encode(b"0" * 64).decode())

    def test_post_probe_byte_and_file_set_mutations_are_rejected(self):
        receipt = {"packageDirectory": str(self.package), **archive_comparison(self.package, *self.packet())}
        (self.package / "lib/tsc").write_bytes(b"mutation after archive verification")
        with self.assertRaises(ValueError):
            recheck_files([receipt])
        (self.package / "lib/tsc").write_bytes(self.files["lib/tsc"])
        (self.package / "new.js").write_bytes(b"later file")
        with self.assertRaises(ValueError):
            recheck_files([receipt])


class RegistryControls(unittest.TestCase):
    def setUp(self):
        self.source = {"C": "c" * 40, "H": "a" * 40, "tag": "v0.999.0", "R": "123"}
        self.dist = {"integrity": "sha512-" + base64.b64encode(b"x" * 64).decode(),
                     "attestations": {"url": "https://registry.npmjs.org/-/npm/v1/attestations/inert"}}
        self.payload = {"predicateType": "https://slsa.dev/provenance/v1",
            "subject": [{"digest": {"sha512": (b"x" * 64).hex()}}],
            "predicate": {"buildDefinition": {"externalParameters": {"workflow": {
                "ref": "refs/heads/release/v0.999.0", "repository": "https://github.com/ubugeeei-prod/vize",
                "path": ".github/workflows/release.yml"}},
                "resolvedDependencies": [{"digest": {"gitCommit": "a" * 40}}]},
                "runDetails": {"metadata": {"invocationId": "https://github.com/ubugeeei-prod/vize/actions/runs/123/attempts/1"}}}}

    def packet(self, payload=None):
        return {"attestations": [{"bundle": {"dsseEnvelope": {
            "payload": base64.b64encode(json.dumps(payload or self.payload).encode()).decode()}}}]}

    def test_payload_binding_explicitly_does_not_claim_signature_verification(self):
        result = provenance_payload_binding(self.dist, self.source, lambda _: self.packet())
        self.assertTrue(result["payloadBindingChecked"])
        self.assertFalse(result["cryptographicSignatureVerificationPerformed"])

    def test_wrong_source_run_workflow_subject_and_duplicate_are_rejected(self):
        for key in ("source", "run", "workflow", "subject"):
            payload = copy.deepcopy(self.payload)
            if key == "source":
                payload["predicate"]["buildDefinition"]["resolvedDependencies"][0]["digest"]["gitCommit"] = "b" * 40
            elif key == "run":
                payload["predicate"]["runDetails"]["metadata"]["invocationId"] = "https://github.com/ubugeeei-prod/vize/actions/runs/124/attempts/1"
            elif key == "workflow":
                payload["predicate"]["buildDefinition"]["externalParameters"]["workflow"]["ref"] = "refs/heads/main"
            else:
                payload["subject"] = []
            with self.subTest(key=key), self.assertRaises(ValueError):
                provenance_payload_binding(self.dist, self.source, lambda _: self.packet(payload))
        packet = self.packet()
        packet["attestations"] *= 2
        with self.assertRaises(ValueError):
            provenance_payload_binding(self.dist, self.source, lambda _: packet)

    def test_missing_provenance_and_external_urls_are_rejected(self):
        with self.assertRaises(ValueError):
            provenance_payload_binding(self.dist, self.source, lambda _: {"attestations": []})
        for url in ("http://registry.npmjs.org/a", "https://registry.npmjs.org.evil/a",
                    "https://registry.npmjs.org:443/a", "https://user@registry.npmjs.org/a"):
            with self.assertRaises(ValueError):
                public_url(url)
        for sri in ("sha256-abc", "sha512-abc", "sha512-" + base64.b64encode(b"x" * 63).decode()):
            with self.assertRaises(ValueError):
                sri_sha512(sri)

    def test_public_metadata_cannot_change_name_version_url_or_sri(self):
        entry = {"resolved": "https://registry.npmjs.org/vize/-/vize-0.999.0.tgz",
                 "integrity": self.dist["integrity"]}
        original = {"name": "vize", "version": "0.999.0",
                    "dist": {"tarball": entry["resolved"], "integrity": entry["integrity"]}}
        for field in ("name", "version", "tarball", "integrity"):
            metadata = copy.deepcopy(original)
            (metadata if field in ("name", "version") else metadata["dist"])[field] = "different"
            with self.assertRaises(ValueError):
                package_metadata("vize", "0.999.0", entry, self.source, lambda _: metadata)

    def test_whole_package_and_provenance_reader_binding_is_inert(self):
        entry = {"resolved": "https://registry.npmjs.org/vize/-/vize-0.999.0.tgz",
                 "integrity": self.dist["integrity"]}
        metadata = {"name": "vize", "version": "0.999.0",
                    "dist": {**self.dist, "tarball": entry["resolved"]}}
        requests = []
        def reader(url):
            requests.append(url)
            return self.packet() if url == self.dist["attestations"]["url"] else metadata
        receipt = package_metadata("vize", "0.999.0", entry, self.source, reader)
        self.assertTrue(receipt["payloadBindingChecked"])
        self.assertFalse(receipt["cryptographicSignatureVerificationPerformed"])
        self.assertEqual(requests, ["https://registry.npmjs.org/vize/0.999.0", self.dist["attestations"]["url"]])
        for source in ({**self.source, "H": "b" * 40}, {**self.source, "R": "124"}):
            with self.assertRaises(ValueError):
                package_metadata("vize", "0.999.0", entry, source, reader)


class IdentityControls(unittest.TestCase):
    def test_nine_initial_overrides_are_rejected_before_node_probe(self):
        for key in OVERRIDES:
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, key):
                reject_overrides({key: "nonempty"})
        reject_overrides({key: "" for key in OVERRIDES})

    def test_output_symlink_and_each_existing_receipt_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            parent = pathlib.Path(temporary).resolve()
            for filename in fresh_outputs(str(parent / "receipt.json")):
                filename.write_text("preserved evidence")
                with self.assertRaises(ValueError):
                    fresh_outputs(str(parent / "receipt.json"))
                self.assertEqual(filename.read_text(), "preserved evidence")
                filename.unlink()
            link = parent / "link"
            link.symlink_to(parent, target_is_directory=True)
            with self.assertRaises(ValueError):
                fresh_outputs(str(link / "receipt.json"))
            with self.assertRaises(ValueError):
                exact_path(str(link), directory=True)

    def test_reviewed_collector_digest_is_required(self):
        authority = producer_authority(ROOT)
        self.assertEqual(producer_authority(ROOT, authority["sha256"]), authority)
        with self.assertRaises(ValueError):
            producer_authority(ROOT, "0" * 64)

    def test_same_size_same_mtime_stale_owned_bytecode_never_executes(self):
        with tempfile.TemporaryDirectory() as temporary:
            copied_root = pathlib.Path(temporary).resolve()
            for name in PRODUCER_FILES:
                destination = copied_root / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / name, destination)
            identity = copied_root / "tools/support/release/public_install/identity.py"
            original = identity.read_bytes()
            mtime = int(identity.stat().st_mtime)
            sentinel = copied_root / "stale-bytecode-executed"
            stale = ("from pathlib import Path\nPath(" + repr(str(sentinel)) + ").write_text('stale')\n").encode()
            identity.write_bytes(stale + b"#" * (len(original) - len(stale)))
            os.utime(identity, (mtime, mtime))
            py_compile.compile(str(identity), doraise=True)
            identity.write_bytes(original)
            os.utime(identity, (mtime, mtime))
            command = [sys.executable, "-I", str(identity.parent / "collect.py"), "--print-collector-authority"]
            result = subprocess.run(command, text=True, capture_output=True, check=True)
            self.assertFalse(sentinel.exists(), "official producer must execute the reviewed .py source")
            self.assertEqual(json.loads(result.stdout), producer_authority(copied_root))
            # Demonstrate that the control really has a matching stale pyc.
            subprocess.run([sys.executable, "-I", "-c", "import sys; sys.path.insert(0, "
                            + repr(str(identity.parent)) + "); import identity"], check=True)
            self.assertEqual(sentinel.read_text(), "stale")



class JournalControls(unittest.TestCase):
    def test_attempt_failed_wrong_pid_and_corsa_never_grant_return_credit(self):
        with tempfile.TemporaryDirectory() as temporary:
            journal = pathlib.Path(temporary) / "inert-journal.jsonl"
            configuration = {"installRoot": "/inert/install", "nativePath": "/inert/install/vize.node", "nativeSha256": "a" * 64}
            common = {"schema": "vize-public-native-custody-event-v1", "pid": 123}
            initialized = {**common, "event": "initialized", **configuration, "sha256": "a" * 64, "node": "/inert/node"}
            attempt = {**common, "event": "attempt", "expectedNative": True, "actualPath": configuration["nativePath"], "sha256": "a" * 64}
            wrong_route = {**attempt, "event": "returned", "corsaPath": "/source/tsgo"}
            for events in ([initialized, attempt], [initialized, attempt, {**attempt, "event": "failed"}],
                           [initialized, {**attempt, "pid": 999}], [initialized, attempt, wrong_route]):
                journal.write_text("".join(json.dumps(event) + "\n" for event in events))
                with self.assertRaises(ValueError):
                    verify_journal(journal, configuration, pathlib.Path("/inert/node"), 123, "/inert/corsa")


if __name__ == "__main__":
    with patch("urllib.request.urlopen", side_effect=AssertionError("inert controls must not reach the network")):
        unittest.main()
