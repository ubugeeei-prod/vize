import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  FILES,
  hasHostImport,
  lexicalView,
  replay,
  rewriteGroupedImports,
  rewriteHostImports,
} from "../../tools/support/levels/move-host-runtime.ts";

const script = fileURLToPath(
  new URL("../../tools/support/levels/move-host-runtime.ts", import.meta.url),
);

function temporary<T>(run: (root: string) => T): T {
  const root = mkdtempSync(path.join(os.tmpdir(), "host-runtime-move-"));
  try {
    return run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function write(root: string, relative: string, contents: string) {
  const filename = path.join(root, relative);
  mkdirSync(path.dirname(filename), { recursive: true });
  writeFileSync(filename, contents);
}

function compile(source: string) {
  temporary((root) => {
    write(
      root,
      "main.rs",
      `#![allow(unused_imports, dead_code)]
mod vize_l0 { pub fn cstr() {} }
mod vize_carton {
    pub mod corsa_api_mode {}
    pub mod corsa_resolver {
        pub fn resolve() {}
        pub mod nested { pub fn find() {} }
    }
}
${source}
fn main() {}
`,
    );
    execFileSync(
      "rustc",
      ["--edition=2024", path.join(root, "main.rs"), "-o", path.join(root, "binary")],
      {
        stdio: "pipe",
      },
    );
  });
}

function movedFiles(root: string) {
  for (const relative of FILES) write(root, `crates/vize_carton/src/${relative}`, "");
}

test("host import aliases survive and the rewritten Rust compiles", () => {
  const source = "use vize_l0::{corsa_resolver::resolve as resolve_host, cstr};";
  const result = rewriteGroupedImports(source);
  assert.ok(result.includes("use vize_carton::corsa_resolver::resolve as resolve_host;"));
  assert.ok(result.includes("use vize_l0::{cstr};"));
  assert.equal(hasHostImport(result), false);
  compile(result);
  assert.equal(rewriteGroupedImports(result), result);
});

test("bare modules, deeply nested groups and comments compile after replay", () => {
  const source = `pub(crate) use vize_l0::{
            corsa_api_mode as mode,
            corsa_resolver::{self as resolver, nested::{find as lookup}},
            cstr // a comment containing } must not end the import
        };`;
  const result = rewriteGroupedImports(source);
  assert.ok(result.includes("corsa_api_mode as mode"));
  assert.ok(result.includes("nested::{find as lookup}"));
  assert.equal(hasHostImport(result), false);
  compile(result);
});

test("check rejects unmigrated direct paths and grouped members", () => {
  const cases = [
    "use vize_l0::corsa_resolver::resolve;",
    "use vize_l0::{corsa_resolver::resolve, cstr};",
    "use vize_l0::{corsa_api_mode, corsa_resolver};",
    "use vize_l0::{corsa_api_mode as mode};",
    "pub use vize_l0::{corsa_resolver::{nested::{find as lookup}}};",
  ];
  temporary((root) => {
    movedFiles(root);
    for (const source of cases) {
      write(root, "crates/vize/src/lib.rs", source);
      assert.throws(() => replay("check", root), /Unmigrated host import/u);
    }
  });
});

test("unrelated storage groups and migrated host aliases remain unchanged", () => {
  for (const source of [
    "use vize_l0::{other::{corsa_resolver}, cstr};",
    "use vize_carton::corsa_resolver as resolver;",
    "use vize_l0::{cstr};",
  ]) {
    assert.equal(hasHostImport(source), false);
    assert.equal(rewriteGroupedImports(source), source);
  }
});

test("import examples in Rust comments and literals remain data", () => {
  const source = String.raw`// use vize_l0::{corsa_resolver, cstr};
/* outer /* use vize_l0::corsa_resolver; */
use vize_l0::{corsa_api_mode};
*/
const RAW: &str = r###"
use vize_l0::{corsa_resolver, cstr};
extern crate vize_l0 as vize_carton;
vize_l0::corsa_api_mode
"###;
const BYTES: &[u8] = br#"
use vize_l0::{corsa_api_mode};
"#;
const C: &std::ffi::CStr = cr#"
use vize_l0::{corsa_resolver};
"#;
const NORMAL: &str = "escaped \" quote
use vize_l0::{corsa_resolver};";
const QUOTE: char = '"';
const ESCAPED: char = '\'';
const UNICODE: char = '\u{22}';
const NON_BMP: char = '🎨';
fn borrowed<'a>(value: &'a str) -> &'a str { value }
`;
  assert.equal(lexicalView(source).length, source.length);
  assert.equal(hasHostImport(source), false);
  assert.equal(rewriteHostImports(source), source);
  compile(source);
  const result = rewriteHostImports(source + "use vize_l0::corsa_resolver::resolve as host;\n");
  assert.ok(result.startsWith(source));
  assert.equal(hasHostImport(result), false);
  compile(result);
});

test("every split import preserves all conditional attributes", () => {
  const source = `#[cfg_attr(all(), cfg(any()))]
#[cfg(not(all()))]
pub(crate) use vize_l0::{missing_storage, corsa_api_mode::missing, corsa_resolver::missing};`;
  const result = rewriteHostImports(source);
  assert.equal(result.split("#[cfg_attr(all(), cfg(any()))]").length - 1, 3);
  assert.equal(result.split("#[cfg(not(all()))]").length - 1, 3);
  assert.equal(hasHostImport(result), false);
  compile(result);
  assert.equal(rewriteHostImports(result), result);
  const inline = rewriteHostImports(
    "# [cfg(any())] use vize_l0::{missing_storage, corsa_resolver::missing};",
  );
  assert.equal(inline.split("# [cfg(any())]").length - 1, 2);
  assert.equal(hasHostImport(inline), false);
  compile(inline);
});

test("real integration and CLI checking preserve literal examples", () => {
  const source = `const EXAMPLE: &str = r#"
use vize_l0::{corsa_resolver, cstr};
extern crate vize_l0 as vize_carton;
vize_l0::corsa_api_mode
"#;
`;
  temporary((root) => {
    movedFiles(root);
    write(root, "davinci/vize_l0/src/lib.rs", "");
    write(root, "davinci/vize_l0/Cargo.toml", "[dependencies]\n");
    write(root, "crates/vize_carton/src/lib.rs", "");
    write(root, "crates/vize_carton/Cargo.toml", "[features]\n");
    for (const consumer of ["vize", "vize_canon", "vize_maestro", "vize_patina"]) {
      write(root, `crates/${consumer}/Cargo.toml`, "[dependencies]\n");
      write(root, `crates/${consumer}/src/lib.rs`, source);
    }
    for (let iteration = 0; iteration < 2; iteration++) {
      execFileSync(process.execPath, [script, "integrate", "--root", root], { stdio: "pipe" });
      execFileSync(process.execPath, [script, "check", "--root", root], { stdio: "pipe" });
      assert.equal(readFileSync(path.join(root, "crates/vize/src/lib.rs"), "utf8"), source);
    }
  });
});
