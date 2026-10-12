#!/usr/bin/env python3
"""Small adversarial laws for the record-only full Rust archive experiment."""
import copy
import hashlib
import io
import json
import os
import shutil
import subprocess
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

from artifact_payload_files import digest_file, json_bytes
from rust_archive_inventory import inventory
import rust_archive_transport as transport
import rust_archive_workers as workers


def original_receipt(archive_sha):
    return {"schemaVersion": 3, "workspaceRoot": transport.WORKSPACE,
            "sha": transport.SOURCE, "tree": transport.TREE, "platform": "linux", "arch": "x64",
            "nextestVersion": "0.9.146", "cargoProfile": "ci", "requireTsgo": None,
            "disableTsgo": "1", "nuxtIterations": "100", "archiveSha256": archive_sha,
            "rustcVersion": "rustc 1.99.0 (b940084d7 2026-09-28)\nbinary: rustc\ncommit-hash: "
            "b940084d7eb6a299eb4bfeb8e34901bc051e7ac4\ncommit-date: 2026-09-28\n"
            "host: x86_64-unknown-linux-gnu\nrelease: 1.99.0\nLLVM version: 23.1.1"}


class Controls(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.zstd = shutil.which("zstd")
        self.assertIsNotNone(self.zstd, "real zstd is required for these controls")
        raw = io.BytesIO()
        with tarfile.open(fileobj=raw, mode="w") as tar:
            for name, body, mode in [("one", b"plumless", 0o755), ("two", b"buckeroo", 0o700),
                                     (".hidden\nraw", b"\0stderr\xffstdout", 0o644)]:
                entry = tarfile.TarInfo("target/ci/" + name)
                entry.size, entry.mode = len(body), mode
                tar.addfile(entry, io.BytesIO(body))
            entry = tarfile.TarInfo("target/empty")
            entry.type, entry.mode = tarfile.DIRTYPE, 0o750
            tar.addfile(entry)
        self.raw = raw.getvalue() + b"\0" * 1024
        self.archive = self.root / "original.tar.zst"
        self.compress(self.raw, self.archive)
        self.before, self.summary = inventory(self.archive, self.root / "before.json", self.zstd)

    def compress(self, body, path):
        path.write_bytes(subprocess.check_output([self.zstd, "-3", "-T1", "-c"], input=body))

    def custody(self):
        custody = self.root / "custody"
        custody.mkdir()
        for arm in ("baseline", "candidate"):
            (custody / f"{arm}-inventory.json").write_bytes(json_bytes(self.before))
        receipt = original_receipt(digest_file(self.archive))
        (custody / "original-receipt.json").write_bytes(json_bytes(receipt))
        summary = {**self.summary, "inventorySha256": digest_file(custody / "baseline-inventory.json")}
        report = {"schemaVersion": 1, "driverSha": "a" * 40, "artifactId": transport.ARTIFACT,
                  "sourceSha": transport.SOURCE, "sourceTree": transport.TREE,
                  "apiSourceSha": transport.API_SOURCE, "originalZipSha256": transport.ZIP_SHA,
                  "originalZipBytes": transport.ZIP_BYTES, "baseline": summary, "candidate": summary,
                  "driverFiles": transport.driver_files(Path(__file__).parent),
                  "encoder": {"command": transport.ENCODER}}
        (custody / "transport.json").write_bytes(json_bytes(report))
        return custody, report

    def test_whole_tar_members_modes_distinct_bodies_and_zero_trailer(self):
        candidate = self.root / "candidate.tar.zst"
        transport.transcode(self.archive, candidate, self.before["tarSha256"], self.zstd)
        after, _ = inventory(candidate, self.root / "after.json", self.zstd)
        self.assertEqual(after, self.before)
        self.assertEqual(self.before["tarSha256"], hashlib.sha256(self.raw).hexdigest())
        self.assertEqual(self.before["tarBytes"], len(self.raw))
        self.assertEqual([member["mode"] for member in after["members"]], [0o755, 0o700, 0o644, 0o750])
        self.assertNotEqual(after["members"][0]["bodySha256"], after["members"][1]["bodySha256"])
        shorter = self.root / "shorter.tar.zst"
        self.compress(self.raw[:-1024], shorter)
        reduced, _ = inventory(shorter, self.root / "reduced.json", self.zstd)
        self.assertEqual(reduced["members"], self.before["members"])
        self.assertNotEqual(reduced["tarSha256"], self.before["tarSha256"])

    def test_truncated_and_corrupted_complete_frame_refused(self):
        body = self.archive.read_bytes()
        for index, altered in enumerate((body[:-3], body[:-1] + bytes([body[-1] ^ 1]))):
            path = self.root / f"corrupt-{index}.zst"
            path.write_bytes(altered)
            with self.assertRaises((ValueError, tarfile.TarError)):
                inventory(path, self.root / f"corrupt-{index}.json", self.zstd)

    def test_original_zip_full_digest_crc_inventory_and_fresh_restore(self):
        path = self.root / "original.zip"
        with zipfile.ZipFile(path, "w") as archive:
            archive.write(self.archive, "tests.tar.zst")
            archive.writestr("receipt.json", json_bytes(original_receipt(digest_file(self.archive))))
        with patch.object(transport, "ZIP_BYTES", path.stat().st_size), \
             patch.object(transport, "ZIP_SHA", digest_file(path)), \
             patch.object(transport, "ARCHIVE_SHA", digest_file(self.archive)):
            destination = self.root / "nested" / "built"
            transport.unpack_original(path, destination)
            self.assertEqual(digest_file(destination / "tests.tar.zst"), digest_file(self.archive))
            with self.assertRaises(ValueError):
                transport.unpack_original(path, destination)
            with zipfile.ZipFile(path, "a") as archive:
                archive.writestr("unexpected.raw", b"opaque")
            with self.assertRaises(ValueError):
                transport.unpack_original(path, self.root / "unexpected")
            with patch.object(transport, "ZIP_BYTES", path.stat().st_size), \
                 patch.object(transport, "ZIP_SHA", digest_file(path)):
                with self.assertRaises(ValueError):
                    transport.unpack_original(path, self.root / "rehash-unexpected")

    def test_receipt_source_profile_runtime_and_compiler_drift_refused(self):
        receipt = original_receipt(transport.ARCHIVE_SHA)
        transport.validate_receipt(receipt)
        for key, value in [("sha", "0" * 40), ("tree", "0" * 40), ("cargoProfile", "dev"),
                           ("disableTsgo", None), ("requireTsgo", "1"), ("nextestVersion", "0.9.145"),
                           ("rustcVersion", "rustc 1.98.0"), ("nuxtIterations", "1")]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                transport.validate_receipt({**receipt, key: value})

    def test_transport_body_inventory_driver_and_member_drift_refused(self):
        custody, report = self.custody()
        with patch.object(transport, "ARCHIVE_SHA", digest_file(self.archive)):
            transport.verify(custody, "candidate", self.archive, "a" * 40)
            with self.assertRaises(ValueError):
                transport.verify(custody, "candidate", self.archive, "b" * 40)
            for key, value in [("sourceSha", "b" * 40), ("artifactId", 1), ("originalZipSha256", "0" * 64)]:
                (custody / "transport.json").write_bytes(json_bytes({**report, key: value}))
                with self.assertRaises(ValueError):
                    transport.verify(custody, "candidate", self.archive, "a" * 40)
            (custody / "transport.json").write_bytes(json_bytes(report))
            changed = copy.deepcopy(self.before)
            changed["members"][0]["mode"] ^= 0o111
            (custody / "candidate-inventory.json").write_bytes(json_bytes(changed))
            mutated = copy.deepcopy(report)
            mutated["candidate"]["inventorySha256"] = digest_file(custody / "candidate-inventory.json")
            (custody / "transport.json").write_bytes(json_bytes(mutated))
            with self.assertRaises(ValueError):
                transport.verify(custody, "candidate", self.archive, "a" * 40)

    def worker_fixture(self):
        root = self.root / "workers"
        root.mkdir()
        for arm in ("baseline", "candidate"):
            for shard in range(1, 5):
                folder = root / f"{arm}-{shard}"
                folder.mkdir()
                listing = {"rust-suites": {"binary": {"binary-id": "binary", "testcases": {
                    f"case-{number}": {"ignored": False, "filter-match": {
                        "status": "matches" if number == shard else "mismatch"}}
                    for number in range(1, 5)}}}}
                listing["rust-suites"]["binary"]["testcases"]["original-ignored"] = {
                    "ignored": True, "filter-match": {"status": "mismatch", "reason": "ignored"}}
                (folder / "tests.json").write_bytes(json_bytes(listing))
                if shard == 1:
                    for case in listing["rust-suites"]["binary"]["testcases"].values():
                        if not case["ignored"]:
                            case["filter-match"]["status"] = "matches"
                    (folder / "unpartitioned.json").write_bytes(json_bytes(listing))
                (folder / "source-results").mkdir()
                (folder / workers.JUNIT).write_text(f'<testsuites><testsuite name="binary"><testcase classname="binary" name="case-{shard}"/></testsuite></testsuites>')
                receipt = {"schemaVersion": 1, "driverSha": "a" * 40, "artifactId": transport.ARTIFACT,
                           "sourceSha": transport.SOURCE, "profile": "pr", "partition": f"hash:{shard}/4",
                           "arm": arm, "shard": shard, "runExit": 0, "tarSha256": self.before["tarSha256"],
                           "runtime": {"sourceSha": transport.SOURCE, "sourceTree": transport.TREE,
                                       "testGitHubSha": transport.SOURCE},
                           "driverFiles": transport.driver_files(Path(__file__).parent),
                           "testsSha256": digest_file(folder / "tests.json"),
                           "junitSha256": digest_file(folder / workers.JUNIT)}
                (folder / "worker.json").write_bytes(json_bytes(receipt))
        return root

    def test_all_four_matched_original_partitions_required(self):
        root = self.worker_fixture()
        self.assertEqual(workers.aggregate(root, "a" * 40)["selectedCases"], 4)
        missing = root / "candidate-4" / "worker.json"
        body = missing.read_bytes()
        missing.unlink()
        with self.assertRaises(ValueError):
            workers.aggregate(root, "a" * 40)
        missing.write_bytes(body)
        receipt = json.loads(body)
        missing.write_bytes(json_bytes({**receipt, "runExit": 1}))
        with self.assertRaises(ValueError):
            workers.aggregate(root, "a" * 40)
        missing.write_bytes(body)
        listing = root / "candidate-4" / "tests.json"
        listing.write_bytes(json_bytes({"rust-suites": {}}))
        missing.write_bytes(json_bytes({**receipt, "testsSha256": digest_file(listing)}))
        with self.assertRaises(ValueError):
            workers.aggregate(root, "a" * 40)

    def test_same_missing_extra_or_duplicate_actual_cases_in_both_arms_refused(self):
        root = self.worker_fixture()
        for xml in ('<testsuite name="binary"/>',
                    '<testsuite name="binary"><testcase classname="binary" name="case-1"/><testcase classname="binary" name="extra"/></testsuite>',
                    '<testsuite name="binary"><testcase classname="binary" name="case-1"/><testcase classname="binary" name="case-1"/></testsuite>',
                    '<testsuite name="binary"><testcase classname="binary" name="case-1"><failure/></testcase></testsuite>',
                    '<testsuite name="binary"><testcase classname="binary" name="case-1"><skipped/></testcase></testsuite>'):
            for arm in ("baseline", "candidate"):
                folder = root / f"{arm}-1"
                (folder / workers.JUNIT).write_text(xml)
                receipt = json.loads((folder / "worker.json").read_bytes())
                receipt["junitSha256"] = digest_file(folder / workers.JUNIT)
                (folder / "worker.json").write_bytes(json_bytes(receipt))
            with self.assertRaises(ValueError):
                workers.aggregate(root, "a" * 40)

    def test_unknown_nextest_release_and_helper_body_mode_drift_refused(self):
        workers.pinned_nextest("cargo-nextest 0.9.146 (x86_64-unknown-linux-gnu)")
        for banner in ("cargo-nextest 0.9.1460", "cargo-nextest 0.9.146-dev", "cargo-nextest 0.9.145"):
            with self.assertRaises(ValueError):
                workers.pinned_nextest(banner)
        custody, report = self.custody()
        for field in ("mode", "sha256"):
            changed = copy.deepcopy(report)
            name = transport.DRIVER_FILES[0]
            changed["driverFiles"][name][field] = 0 if field == "mode" else "0" * 64
            (custody / "transport.json").write_bytes(json_bytes(changed))
            with patch.object(transport, "ARCHIVE_SHA", digest_file(self.archive)), self.assertRaises(ValueError):
                transport.verify(custody, "candidate", self.archive, "a" * 40)

    def test_replacement_objects_cannot_borrow_driver_source_bodies(self):
        repo = self.root / "driver-repo"
        helpers = repo / "tools/support/compat/github"
        helpers.mkdir(parents=True)
        def git(*args):
            return subprocess.check_output(["git", "-C", str(repo), *args], stderr=subprocess.PIPE).decode().strip()
        git("init", "--quiet")
        for name in transport.DRIVER_FILES:
            (helpers / name).write_text("original helper\n")
            (helpers / name).chmod(0o644)
        git("add", ".")
        git("-c", "user.name=Archive control", "-c", "user.email=archive-control@example.invalid",
            "commit", "--quiet", "-m", "original")
        original = git("rev-parse", "HEAD")
        borrowed = helpers / transport.DRIVER_FILES[0]
        borrowed.write_text("borrowed helper\n")
        git("add", ".")
        git("-c", "user.name=Archive control", "-c", "user.email=archive-control@example.invalid",
            "commit", "--quiet", "-m", "borrowed")
        replacement = git("rev-parse", "HEAD")
        git("checkout", "--quiet", "--detach", original)
        git("replace", original, replacement)
        previous = Path.cwd()
        try:
            os.chdir(repo)
            with patch.object(transport, "__file__", str(helpers / "rust_archive_transport.py")):
                transport.authenticate_driver(original)
                borrowed.write_text("borrowed helper\n")
                with self.assertRaises(ValueError):
                    transport.authenticate_driver(original)
        finally:
            os.chdir(previous)

    def test_original_ignored_policy_remains_in_listing_without_forcing_execution(self):
        root = self.worker_fixture()
        self.assertEqual(workers.aggregate(root, "a" * 40)["selectedCases"], 4)
        for arm in ("baseline", "candidate"):
            listing = json.loads((root / f"{arm}-1" / "tests.json").read_bytes())
            ignored = listing["rust-suites"]["binary"]["testcases"]["original-ignored"]
            self.assertEqual(ignored, {"ignored": True, "filter-match": {"status": "mismatch", "reason": "ignored"}})
            self.assertNotIn(("binary", "original-ignored"), workers.selected(listing))
            self.assertEqual(len(workers.outcomes(root / f"{arm}-1" / workers.JUNIT, listing)), 1)

    def test_frozen_test_identity_matches_real_git_without_changing_driver(self):
        repo = self.root / "test-source"
        repo.mkdir()
        def git(*args):
            return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()
        git("init", "--quiet")
        (repo / "fixture").write_text("original provenance check\n")
        git("add", ".")
        git("-c", "user.name=Archive control", "-c", "user.email=archive-control@example.invalid",
            "commit", "--quiet", "-m", "physical source")
        source = git("--no-replace-objects", "rev-parse", "HEAD")
        driver = "d" * 40
        previous = Path.cwd()
        try:
            os.chdir(repo)
            with patch.object(workers, "SOURCE", source), patch.dict(os.environ, {"GITHUB_SHA": driver}):
                environment = workers.frozen_test_environment(driver)
                check = ["python3", "-c", "import os,subprocess; "
                         "head=subprocess.check_output(['git','--no-replace-objects','rev-parse','HEAD'],"
                         "text=True).strip(); assert head==os.environ['GITHUB_SHA']"]
                inherited = subprocess.run(check, capture_output=True)
                self.assertNotEqual(inherited.returncode, 0)
                self.assertEqual(subprocess.run(check, env=environment, capture_output=True).returncode, 0)
                self.assertEqual(os.environ["GITHUB_SHA"], driver)
                with self.assertRaises(ValueError):
                    workers.frozen_test_environment("b" * 40)
            with patch.object(workers, "SOURCE", "b" * 40), patch.dict(os.environ, {"GITHUB_SHA": driver}):
                with self.assertRaises(ValueError):
                    workers.frozen_test_environment(driver)
        finally:
            os.chdir(previous)


if __name__ == "__main__":
    unittest.main()
