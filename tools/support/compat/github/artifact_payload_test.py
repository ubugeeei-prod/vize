import json
import os
import tempfile
import unittest
import warnings
import zipfile
from pathlib import Path

from artifact_payload import MANIFEST, PAYLOAD, RECEIPT, pack, restore
from artifact_payload_files import digest_file, json_bytes, verify_tree

SOURCE = "1" * 40
DRIVER = "2" * 40
ARTIFACT = 42


class PayloadControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="vize-payload-controls-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.original = self.root / "original.zip"
        self.envelope = self.root / "envelope"
        self.expected = {}
        with zipfile.ZipFile(self.original, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for index in range(90):
                self.add(archive, f"complete/case-{index}/native-tsc", b"unchanged native body\x00\xff" * 512, 0o755)
                self.add(archive, f"complete/case-{index}/stdout.raw", bytes([index, 0, 255, 13, 10]))
            # Distinct historical binaries must survive, regardless of similar names.
            for index in range(3):
                self.add(archive, f"historical-{index}/source-native.node", bytes([index]) * 2048)
            self.add(archive, "雪/.hidden/name\nnewline", b"opaque\x00\x80\xff\r\n", 0o600)
            self.add(archive, "stderr.raw", b"")
            # Different equal-size bodies with the same CRC32 must remain distinct.
            self.add(archive, "crc-collision-a.bin", b"plumless")
            self.add(archive, "crc-collision-b.bin", b"buckeroo")
            directory = zipfile.ZipInfo("empty/")
            directory.external_attr = (0o040750 << 16) | 0x10
            archive.writestr(directory, b"")
        self.metadata = {
            "id": ARTIFACT, "name": "complete-original", "size_in_bytes": self.original.stat().st_size,
            "digest": f"sha256:{digest_file(self.original)}",
            "workflow_run": {"head_sha": SOURCE, "id": 123},
        }

    def add(self, archive, name, body, mode=0o644):
        info = zipfile.ZipInfo(name)
        info.external_attr = (0o100000 | mode) << 16
        info.compress_type = zipfile.ZIP_DEFLATED
        archive.writestr(info, body)
        self.expected[name] = (body, mode)

    def packed(self):
        return pack(self.original, self.envelope, self.metadata, SOURCE, DRIVER)

    def restored(self, name="restored"):
        destination = self.root / name
        restore(self.envelope, destination, SOURCE, DRIVER, ARTIFACT)
        return destination

    def receipt(self):
        return json.loads((self.envelope / RECEIPT).read_bytes())

    def repack_objects(self, missing=None, changed=None):
        original = self.envelope / PAYLOAD
        replaced = self.root / "replaced.zip"
        with zipfile.ZipFile(original) as source, zipfile.ZipFile(replaced, "w") as target:
            for info in source.infolist():
                if info.filename == missing:
                    continue
                body = source.read(info)
                target.writestr(info, b"different whole body" if info.filename == changed else body)
        replaced.replace(original)
        receipt = self.receipt()
        receipt["payloadZipSha256"] = digest_file(original)
        (self.envelope / RECEIPT).write_bytes(json_bytes(receipt))

    def test_complete_original_and_independent_mutation(self):
        before = digest_file(self.original)
        receipt = self.packed()
        restored = self.restored()
        manifest = json.loads((self.envelope / MANIFEST).read_bytes())
        self.assertEqual(receipt["fileCount"], len(self.expected))
        self.assertEqual(receipt["entryCount"], len(self.expected) + 1)
        self.assertLess(receipt["uniqueBodyBytes"], receipt["rawBytes"])
        native_rows = [row for row in manifest["entries"] if row["path"].endswith("native-tsc")]
        self.assertEqual(len(native_rows), 90)
        self.assertEqual(len({row["sha256"] for row in native_rows}), 1)
        historical = [row for row in manifest["entries"] if row["path"].endswith("source-native.node")]
        self.assertEqual(len({row["sha256"] for row in historical}), 3)
        collisions = [row for row in manifest["entries"] if row["path"].startswith("crc-collision-")]
        self.assertEqual(len({row["sha256"] for row in collisions}), 2)
        with zipfile.ZipFile(self.original) as archive:
            a, b = (archive.getinfo(f"crc-collision-{name}.bin") for name in ("a", "b"))
            self.assertEqual((a.CRC, a.file_size), (b.CRC, b.file_size))
        for name, (body, mode) in self.expected.items():
            self.assertEqual((restored / name).read_bytes(), body)
            self.assertEqual((restored / name).stat().st_mode & 0o7777, mode)
            self.assertEqual((restored / name).stat().st_nlink, 1)
        self.assertEqual((restored / "empty").stat().st_mode & 0o7777, 0o750)
        first = restored / "complete/case-0/native-tsc"
        last = restored / "complete/case-89/native-tsc"
        self.assertNotEqual(first.stat().st_ino, last.stat().st_ino)
        first.write_bytes(b"one independent mutation")
        self.assertEqual(last.read_bytes(), self.expected["complete/case-89/native-tsc"][0])
        with self.assertRaisesRegex(ValueError, "restored file body differs"):
            verify_tree(restored, manifest)
        self.assertEqual(digest_file(self.original), before)

    def test_missing_changed_extra_modes_and_external_aliases(self):
        self.packed()
        manifest = json.loads((self.envelope / MANIFEST).read_bytes())
        for control in ("missing", "extra", "mode", "symlink", "hardlink"):
            with self.subTest(control=control):
                destination = self.restored(control)
                first = destination / "complete/case-0/native-tsc"
                if control == "missing":
                    first.unlink()
                elif control == "extra":
                    (destination / "extra").write_bytes(b"unexpected")
                elif control == "mode":
                    first.chmod(0o644)
                elif control == "symlink":
                    first.unlink()
                    first.symlink_to(self.original)
                else:
                    alias = destination / "complete/case-89/native-tsc"
                    alias.unlink()
                    os.link(first, alias)
                with self.assertRaises(ValueError):
                    verify_tree(destination, manifest)

    def test_unreadable_directory_cannot_hide_extra_inventory(self):
        directory = self.root / "permission-control"
        directory.mkdir(mode=0o755)
        closed = directory / "empty"
        closed.mkdir()
        (closed / "unexpected.raw").write_bytes(b"opaque unobserved bytes\x00\xff")
        closed.chmod(0)
        manifest = {"entries": [{"path": "empty", "kind": "directory", "mode": 0}]}
        try:
            if os.geteuid() == 0:
                # A root runner can read mode000: exercise the real refusal as nobody.
                self.root.chmod(0o755)
                child = os.fork()
                if child == 0:
                    try:
                        os.setgroups([])
                        os.setgid(65534)
                        os.setuid(65534)
                        verify_tree(directory, manifest)
                    except ValueError:
                        os._exit(0)
                    except BaseException:
                        os._exit(2)
                    os._exit(1)
                _, status = os.waitpid(child, 0)
                self.assertEqual(os.waitstatus_to_exitcode(status), 0,
                                 "unreadable inventory must refuse under an unprivileged identity")
            else:
                with self.assertRaisesRegex(ValueError, "cannot inspect complete restored inventory"):
                    verify_tree(directory, manifest)
        finally:
            closed.chmod(0o755)

    def test_corrupt_archive_and_whole_object(self):
        self.packed()
        with zipfile.ZipFile(self.envelope / PAYLOAD) as archive:
            name = archive.infolist()[0].filename
        self.repack_objects(changed=name)
        with self.assertRaisesRegex(ValueError, "damaged whole object body"):
            self.restored("whole-body-corruption")
        archive = self.envelope / PAYLOAD
        with archive.open("r+b") as stream:
            stream.write(b"corrupt archive bytes")
        with self.assertRaisesRegex(ValueError, "damaged payload archive"):
            self.restored("transport-corruption")

    def test_missing_object_with_recomputed_outer_digest(self):
        self.packed()
        with zipfile.ZipFile(self.envelope / PAYLOAD) as archive:
            name = archive.infolist()[0].filename
        self.repack_objects(missing=name)
        with self.assertRaisesRegex(ValueError, "missing or unexpected payload object"):
            self.restored()

    def test_source_driver_and_original_digest_are_bound(self):
        self.packed()
        for source, driver, artifact in ((DRIVER, DRIVER, ARTIFACT), (SOURCE, SOURCE, ARTIFACT),
                                         (SOURCE, DRIVER, ARTIFACT + 1)):
            with self.assertRaisesRegex(ValueError, "transport source or driver identity differs"):
                restore(self.envelope, self.root / "wrong-identity", source, driver, artifact)
        bad_metadata = {**self.metadata, "digest": "sha256:" + "0" * 64}
        with self.assertRaisesRegex(ValueError, "original archive digest differs"):
            pack(self.original, self.root / "wrong-original", bad_metadata, SOURCE, DRIVER)

    def test_manifest_corruption_and_external_envelope_symlink(self):
        self.packed()
        manifest = self.envelope / MANIFEST
        original_bytes = manifest.read_bytes()
        manifest.write_bytes(original_bytes + b" ")
        with self.assertRaisesRegex(ValueError, "damaged payload manifest"):
            self.restored("wrong-manifest")
        manifest.unlink()
        external = self.root / "external.json"
        external.write_bytes(original_bytes)
        manifest.symlink_to(external)
        with self.assertRaisesRegex(ValueError, "invalid envelope file"):
            self.restored("external-manifest")
        external_tree = self.root / "external-tree"
        external_tree.mkdir()
        borrowed = self.root / "borrowed-root"
        borrowed.symlink_to(external_tree, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "restored root must be an independent directory"):
            verify_tree(borrowed, {"entries": []})

    def test_unsafe_duplicate_and_symlink_original_entries_fail_closed(self):
        for control in ("../escape", "/absolute", "duplicate", "symlink"):
            with self.subTest(control=control), warnings.catch_warnings():
                warnings.simplefilter("ignore", UserWarning)
                original = self.root / f"control-{control.replace('/', '_')}.zip"
                with zipfile.ZipFile(original, "w") as archive:
                    info = zipfile.ZipInfo(control)
                    if control == "symlink":
                        info.external_attr = 0o120777 << 16
                    archive.writestr(info, b"original")
                    if control == "duplicate":
                        archive.writestr(info, b"second different body")
                metadata = {**self.metadata, "size_in_bytes": original.stat().st_size,
                            "digest": f"sha256:{digest_file(original)}"}
                with self.assertRaises(ValueError):
                    pack(original, self.root / f"output-{control.replace('/', '_')}",
                         metadata, SOURCE, DRIVER)


if __name__ == "__main__":
    unittest.main()
