"""Raw source-object and genuine committed manifest-shape rejection laws."""

import copy
import json
import pathlib
import re
import runpy
import subprocess
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[4]
sys.dont_write_bytecode = True
COLLECTOR = runpy.run_path(str(ROOT / "tools/support/release/public_install/collect.py"))
from identity import producer_authority, source_file, source_identity, source_manifests


class SourceControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = pathlib.Path(self.temporary.name).resolve()
        self.git("init", "-q")
        self.git("commit", "--allow-empty", "-qm", "inert C")
        self.cut = self.git("rev-parse", "HEAD")
        self.actual_head = subprocess.check_output(["git", "--no-replace-objects", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        self.paths = ("npm/cli/package.json", "npm/oxlint/package.json", "npm/native/package.json")
        self.original = {name: json.loads(source_file(ROOT, self.actual_head, name)) for name in self.paths}
        self.workspace = source_file(ROOT, self.actual_head, "pnpm-workspace.yaml").decode()
        self.wrapper = source_file(ROOT, self.actual_head, "npm/cli/bin/vize")
        self.write_shape("0.999.0")

    def git(self, *args):
        return subprocess.check_output(["git", "-c", "user.name=Inert test", "-c",
            "user.email=inert@example.invalid", *args], cwd=self.root, text=True, stderr=subprocess.PIPE).strip()

    def write_shape(self, version):
        for name, original in self.original.items():
            manifest = copy.deepcopy(original)
            manifest["version"] = version
            destination = self.root / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_text(json.dumps(manifest))
        workspace = re.sub(r'^(    "@vizejs/native-[^"]+": ")[^"]+(".*)$',
                           lambda match: match[1] + version + match[2], self.workspace, flags=re.M)
        (self.root / "pnpm-workspace.yaml").write_text(workspace)
        wrapper = self.root / "npm/cli/bin/vize"
        wrapper.parent.mkdir(parents=True, exist_ok=True)
        wrapper.write_bytes(self.wrapper)

    def commit(self):
        self.git("add", ".")
        self.git("commit", "-qm", "inert committed manifest shape")
        return self.git("rev-parse", "HEAD")

    def source(self, head, version="0.999.0"):
        return {"H": head, "tag": "v" + version}

    def test_genuine_committed_cli_workspace_and_corsa_shape_is_retained(self):
        version = self.original["npm/cli/package.json"]["version"]
        receipt = source_manifests(ROOT, self.source(self.actual_head, version))
        self.assertEqual(len(receipt["files"]), 4)
        self.assertGreaterEqual(int(receipt["corsaVersion"].split(".")[0]), 7)
        self.assertNotIn("@vizejs/native-darwin-arm64", self.original["npm/cli/package.json"]["optionalDependencies"])
        head = self.commit()
        self.assertEqual(len(source_manifests(self.root, self.source(head))["files"]), 4)

    def test_wrong_h_versions_native_edges_platform_and_corsa_refuse(self):
        mutations = (
            ("npm/cli/package.json", "version", None, "0.998.0"),
            ("npm/native/package.json", "version", None, "0.998.0"),
            ("npm/cli/package.json", "dependencies", "@vizejs/native", "workspace:^"),
            ("npm/cli/package.json", "dependencies", "@vizejs/native", "0.998.0"),
            ("npm/cli/package.json", "optionalDependencies", "@typescript/typescript-darwin-arm64", "catalog:wrong"),
            ("npm/oxlint/package.json", "optionalDependencies", "@vizejs/native-darwin-arm64", "0.998.0"),
        )
        for name, field, child, value in mutations:
            self.write_shape("0.999.0")
            path = self.root / name
            manifest = json.loads(path.read_bytes())
            if child:
                manifest[field][child] = value
            else:
                manifest[field] = value
            path.write_text(json.dumps(manifest))
            with self.subTest(name=name, field=field), self.assertRaises(ValueError):
                source_manifests(self.root, self.source(self.commit()))
        corsa_version = source_manifests(ROOT, self.source(self.actual_head, self.original["npm/cli/package.json"]["version"]))["corsaVersion"]
        for before, after in (("0.999.0", "0.998.0"), ('"' + corsa_version + '"', '"6.9.0"')):
            self.write_shape("0.999.0")
            workspace = self.root / "pnpm-workspace.yaml"
            self.assertIn(before, workspace.read_text())
            workspace.write_text(workspace.read_text().replace(before, after))
            with self.assertRaises(ValueError):
                source_manifests(self.root, self.source(self.commit()))

    def test_full_identity_and_exact_raw_single_parent_are_required(self):
        head = self.commit()
        self.assertEqual(source_identity(self.root, self.cut, head, "v0.999.0", "123")["H"], head)
        for cut, candidate, tag, run in ((head, self.cut, "v0.999.0", "123"),
                (self.cut[:8], head, "v0.999.0", "123"), (self.cut, head, "v0.999.1", "123"),
                (self.cut, head, "v0.999.0", "0")):
            with self.assertRaises(ValueError):
                source_identity(self.root, cut, candidate, tag, run)

    def test_installed_cli_native_and_corsa_must_equal_actual_h_edges_before_probe(self):
        head = self.commit()
        corsa = source_manifests(self.root, self.source(head))["corsaVersion"]
        with tempfile.TemporaryDirectory() as temporary:
            install = pathlib.Path(temporary).resolve()
            versions = {name: "0.999.0" for name in ("vize", "oxlint-plugin-vize", "@vizejs/native", "@vizejs/native-darwin-arm64")}
            (install / "package.json").write_text(json.dumps({"private": True, "dependencies": versions}))
            (install / "package-lock.json").write_text(json.dumps({"lockfileVersion": 3, "packages": {"": {"dependencies": versions}}}))
            manifest_path = install / "node_modules/vize/package.json"
            manifest_path.parent.mkdir(parents=True)
            sentinel = self.root / "node-started"
            node = self.root / "fake-node"
            node.write_text("#!/bin/sh\ntouch '" + str(sentinel) + "'\nexit 99\n")
            node.chmod(0o700)
            args = types.SimpleNamespace(root=str(self.root), cut=self.cut, head=head,
                tag="v0.999.0", run="123", install_root=str(install), node=str(node),
                output=str(self.root / "receipt.json"), collector_sha256=producer_authority(ROOT)["sha256"])
            for edge in ("native", "corsa"):
                manifest = copy.deepcopy(self.original["npm/cli/package.json"])
                manifest["dependencies"]["@vizejs/native"] = "0.998.0" if edge == "native" else "0.999.0"
                manifest["optionalDependencies"]["@typescript/typescript-darwin-arm64"] = "7.999.0" if edge == "corsa" else corsa
                manifest_path.write_text(json.dumps(manifest))
                with self.assertRaisesRegex(ValueError, "actual H declaration"):
                    COLLECTOR["collect"](args)
                self.assertFalse(sentinel.exists())
                self.assertFalse((self.root / "receipt.json").exists())

    def test_git_replacement_cannot_substitute_h_source_or_cli_wrapper(self):
        self.write_shape("0.998.0")
        head = self.commit()
        with self.assertRaises(ValueError):
            source_manifests(self.root, self.source(head))
        self.write_shape("0.999.0")
        wrapper = self.root / "npm/cli/bin/vize"
        wrapper.write_bytes(b"inert replacement wrapper")
        self.git("add", ".")
        replacement = self.git("commit-tree", self.git("write-tree"), "-p", self.cut, "-m", "inert replacement")
        self.git("replace", head, replacement)
        self.assertEqual(self.git("show", head + ":npm/cli/bin/vize"), "inert replacement wrapper")
        self.assertEqual(source_file(self.root, head, "npm/cli/bin/vize"), self.wrapper)
        self.assertEqual(source_identity(self.root, self.cut, head, "v0.999.0", "123")["H"], head)
        with self.assertRaises(ValueError):
            source_manifests(self.root, self.source(head))

    def test_legacy_graft_cannot_hide_a_raw_second_parent(self):
        extra = self.commit()
        head = self.git("commit-tree", self.git("rev-parse", "HEAD^{tree}"),
                        "-p", self.cut, "-p", extra, "-m", "inert two-parent H")
        with self.assertRaises(ValueError):
            source_identity(self.root, self.cut, head, "v0.999.0", "123")
        graft = self.root / ".git/info/grafts"
        graft.write_text(head + " " + self.cut + "\n")
        self.assertEqual(self.git("--no-replace-objects", "rev-list", "--parents", "-n", "1", head), head + " " + self.cut)
        with self.assertRaises(ValueError):
            source_identity(self.root, self.cut, head, "v0.999.0", "123")

    def test_copied_committed_shape_wrong_version_refuses_before_fake_node(self):
        # PR checkouts may be shallow two-parent merge commits. Copy their raw
        # manifest/catalog bytes into owned actual C -> H history for this law.
        # The copied shape is inert control data, never publication authority.
        for name in (*self.paths, "pnpm-workspace.yaml", "npm/cli/bin/vize"):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(source_file(ROOT, self.actual_head, name))
        head = self.commit()
        self.assertEqual(source_identity(self.root, self.cut, head, "v0.999999.0", "123")["H"], head)
        sentinel = self.root / "node-started"
        node = self.root / "fake-node"
        node.write_text("#!/bin/sh\ntouch '" + str(sentinel) + "'\nexit 99\n")
        node.chmod(0o700)
        args = types.SimpleNamespace(root=str(self.root), cut=self.cut, head=head,
            tag="v0.999999.0", run="123", install_root=str(self.root), node=str(node),
            output=str(self.root / "receipt.json"), collector_sha256=producer_authority(ROOT)["sha256"])
        with self.assertRaisesRegex(ValueError, "catalog version"):
            COLLECTOR["collect"](args)
        self.assertFalse(sentinel.exists())
        self.assertFalse((self.root / "receipt.json").exists())

    def test_genuine_two_parent_checkout_is_refused_before_any_probe(self):
        main = self.git("symbolic-ref", "--short", "HEAD")
        self.commit()
        self.git("branch", "inert-side", self.cut)
        self.git("checkout", "-q", "inert-side")
        self.git("commit", "--allow-empty", "-qm", "inert side")
        self.git("checkout", "-q", main)
        self.git("merge", "--no-ff", "inert-side", "-m", "genuine inert checkout merge")
        head = self.git("rev-parse", "HEAD")
        raw = self.git("--no-replace-objects", "cat-file", "commit", head)
        parents = [line[7:] for line in raw.split("\n\n", 1)[0].splitlines() if line.startswith("parent ")]
        self.assertEqual(len(parents), 2)
        sentinel = self.root / "node-started"
        node = self.root / "fake-node"
        node.write_text("#!/bin/sh\ntouch '" + str(sentinel) + "'\nexit 99\n")
        node.chmod(0o700)
        args = types.SimpleNamespace(root=str(self.root), cut=parents[0], head=head,
            tag="v0.999.0", run="123", install_root=str(self.root), node=str(node),
            output=str(self.root / "receipt.json"), collector_sha256=producer_authority(ROOT)["sha256"])
        with self.assertRaisesRegex(ValueError, "exact single-parent"):
            COLLECTOR["collect"](args)
        self.assertFalse(sentinel.exists())
        self.assertFalse((self.root / "receipt.json").exists())


if __name__ == "__main__":
    with patch("urllib.request.urlopen", side_effect=AssertionError("inert controls must not reach the network")):
        unittest.main()
