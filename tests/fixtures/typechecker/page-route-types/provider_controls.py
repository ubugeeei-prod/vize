"""Offline transport controls; these do not execute or fetch a provider."""

import base64
import hashlib
import io
import json
from pathlib import Path
import tarfile
import unittest

import provider


def archive(entries):
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:gz") as target:
        for name, body, kind in entries:
            entry = tarfile.TarInfo(name)
            entry.size = len(body)
            entry.type = kind
            if kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
                entry.linkname = "outside"
            target.addfile(entry, io.BytesIO(body))
    return output.getvalue()


def decode(entries):
    raw = archive(entries)
    integrity = base64.b64encode(hashlib.sha512(raw).digest()).decode()
    return provider.members(raw, integrity, len(entries), sum(len(body) for _, body, _ in entries))


class ProviderTransport(unittest.TestCase):
    def test_regular_files_preserve_complete_original_bytes(self):
        files = decode([
            ("package/LICENSE", b"original license\n", tarfile.REGTYPE),
            ("package/dist/日本語.d.ts", b"export type T = '\xf0\x9f\x8e\xa8';\r\n", tarfile.REGTYPE),
        ])
        self.assertEqual([(name, body) for name, body, _mode in files], [
            ("LICENSE", b"original license\n"),
            ("dist/日本語.d.ts", b"export type T = '\xf0\x9f\x8e\xa8';\r\n"),
        ])

    def test_digest_refuses_modified_complete_archive(self):
        raw = archive([("package/LICENSE", b"license", tarfile.REGTYPE)])
        integrity = base64.b64encode(hashlib.sha512(raw).digest()).decode()
        with self.assertRaises(ValueError):
            provider.members(raw + b"changed", integrity, 1, 7)

    def test_member_count_and_total_refuse_missing_or_added_data(self):
        raw = archive([("package/LICENSE", b"license", tarfile.REGTYPE)])
        integrity = base64.b64encode(hashlib.sha512(raw).digest()).decode()
        for count, total in [(0, 7), (2, 7), (1, 6), (1, 8)]:
            with self.subTest(count=count, total=total), self.assertRaises(ValueError):
                provider.members(raw, integrity, count, total)

    def test_unsafe_members_never_reach_extraction(self):
        for name, kind in [
            ("/package/file", tarfile.REGTYPE),
            ("elsewhere/file", tarfile.REGTYPE),
            ("package/../outside", tarfile.REGTYPE),
            ("package/./file", tarfile.REGTYPE),
            ("package\\outside", tarfile.REGTYPE),
            ("package/link", tarfile.SYMTYPE),
            ("package/hardlink", tarfile.LNKTYPE),
            ("package/device", tarfile.CHRTYPE),
        ]:
            with self.subTest(name=name, kind=kind), self.assertRaises(ValueError):
                decode([(name, b"", kind)])

    def test_duplicate_members_are_refused(self):
        with self.assertRaises(ValueError):
            decode([("package/file", b"first", tarfile.REGTYPE), ("package/file", b"second", tarfile.REGTYPE)])

    def test_existing_or_symlink_destination_refused_before_network(self):
        import tempfile
        with tempfile.TemporaryDirectory() as temporary:
            existing = Path(temporary) / "existing"
            existing.mkdir()
            link = Path(temporary) / "link"
            link.symlink_to(existing, target_is_directory=True)
            for target in [existing, link, Path(temporary) / "missing" / "child"]:
                with self.subTest(target=target), self.assertRaises(ValueError):
                    provider.stage(target)

    def test_required_exports_and_license_are_authoritative(self):
        exports = {".": {"types": "./core.d.ts"}, "./experimental": {"types": "./experimental.d.ts"},
                   "./auto-routes": {"types": "./auto-routes.d.mts"}, "./auto": {"types": "./auto.d.ts"},
                   "./volar/sfc-typed-router": {"default": "./plugin.cjs", "types": "./plugin.d.cts"}}
        package = {"name": "vue-router", "version": "5.1.0", "exports": exports}
        names = ["core.d.ts", "experimental.d.ts", "auto-routes.d.mts", "auto.d.ts", "plugin.cjs", "plugin.d.cts", "LICENSE"]
        files = [("package.json", json.dumps(package).encode(), 0o644)] + [(name, b"original", 0o644) for name in names]
        self.assertEqual(provider.validate_package(files), package)
        for missing in names:
            with self.subTest(missing=missing), self.assertRaises(ValueError):
                provider.validate_package([row for row in files if row[0] != missing])
        package["version"] = "4.5.1"
        with self.assertRaises(ValueError):
            provider.validate_package([(name, json.dumps(package).encode() if name == "package.json" else body, mode) for name, body, mode in files])


if __name__ == "__main__":
    unittest.main()
