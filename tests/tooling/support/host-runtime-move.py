"""Regression laws for host-import replay, including actual Rust syntax."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("host_move", ROOT / "tools/support/levels/move-host-runtime.py")
move = importlib.util.module_from_spec(spec)
spec.loader.exec_module(move)

class HostImportLaws(unittest.TestCase):
    def compile(self, source):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            program = root / "main.rs"
            program.write_text('''#![allow(unused_imports, dead_code)]
mod vize_l0 { pub fn cstr() {} }
mod vize_carton {
    pub mod corsa_api_mode {}
    pub mod corsa_resolver {
        pub fn resolve() {}
        pub mod nested { pub fn find() {} }
    }
}
''' + source + '\nfn main() {}\n')
            result = subprocess.run(["rustc", "--edition=2024", str(program), "-o", str(root / "binary")], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_alias_is_preserved_and_output_compiles(self):
        source = "use vize_l0::{corsa_resolver::resolve as resolve_host, cstr};"
        result = move.rewrite_grouped_imports(source)
        self.assertIn("use vize_carton::corsa_resolver::resolve as resolve_host;", result)
        self.assertIn("use vize_l0::{cstr};", result)
        self.assertFalse(move.has_host_import(result))
        self.compile(result)
        self.assertEqual(move.rewrite_grouped_imports(result), result)

    def test_bare_modules_deep_groups_and_comments_compile(self):
        source = '''pub(crate) use vize_l0::{
            corsa_api_mode as mode,
            corsa_resolver::{self as resolver, nested::{find as lookup}},
            cstr // a comment containing } must not end the import
        };'''
        result = move.rewrite_grouped_imports(source)
        self.assertIn("corsa_api_mode as mode", result)
        self.assertIn("nested::{find as lookup}", result)
        self.assertFalse(move.has_host_import(result))
        self.compile(result)

    def test_check_rejects_unmigrated_paths_and_grouped_members(self):
        cases = [
            "use vize_l0::corsa_resolver::resolve;",
            "use vize_l0::{corsa_resolver::resolve, cstr};",
            "use vize_l0::{corsa_api_mode, corsa_resolver};",
            "use vize_l0::{corsa_api_mode as mode};",
            "pub use vize_l0::{corsa_resolver::{nested::{find as lookup}}};",
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in move.FILES:
                destination = root / "crates/vize_carton/src" / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_text("")
            source = root / "crates/vize/src/lib.rs"
            source.parent.mkdir(parents=True, exist_ok=True)
            with patch.object(move, "ROOT", root), patch.object(sys, "argv", ["move-host-runtime.py", "check"]):
                for case in cases:
                    source.write_text(case)
                    with self.subTest(case=case), self.assertRaisesRegex(SystemExit, "Unmigrated host import"):
                        move.main()

    def test_other_storage_groups_and_host_aliases_are_not_rewritten(self):
        for source in ["use vize_l0::{other::{corsa_resolver}, cstr};", "use vize_carton::corsa_resolver as resolver;", "use vize_l0::{cstr};"]:
            self.assertFalse(move.has_host_import(source))
            self.assertEqual(move.rewrite_grouped_imports(source), source)

if __name__ == "__main__":
    unittest.main()
