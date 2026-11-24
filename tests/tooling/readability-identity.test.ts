import assert from "node:assert/strict";
import { test } from "node:test";

import { compareScriptFile } from "../../tools/commands/levels/ast-identity.ts";
import {
  compareWithLocations,
  normalizeLocations,
} from "../../tools/commands/levels/ir-identity-locations.ts";
import { compareRust } from "../../tools/commands/levels/rust-token-identity.ts";

const rustBase = [
  "/// Walks the tree.",
  "pub fn walk(a: bool, b: bool) -> bool {",
  "    // Both flags must hold (#1234).",
  '    let raw = r#"// not a comment"#;',
  "    a && b && !raw.is_empty()",
  "}",
  "",
].join("\n");

test("Rust token identity ignores plain comments and whitespace", () => {
  const after = rustBase.replace("    // Both flags must hold (#1234).\n", "\n");
  assert.equal(compareRust("a.rs", rustBase, after).verdict, "identical");
  assert.equal(compareRust("a.rs", rustBase, rustBase).verdict, "bytes-identical");
});

test("Rust token identity reports doc, spacing, literal and position changes", () => {
  const verdict = (after: string, before = rustBase) => compareRust("a.rs", before, after).verdict;
  assert.equal(verdict(rustBase.replace("Walks the tree.", "Walk the tree.")), "doc-only");
  const clap = `#[derive(Parser)]\n${rustBase}`;
  assert.equal(verdict(clap.replace("Walks", "Walk"), clap), "doc-product-text");
  assert.equal(verdict(rustBase.replace("a && b", "a & &b")), "different");
  assert.equal(verdict(rustBase.replace("not a comment", "not  a comment")), "different");
  const located = "fn here() -> u32 {\n    line!()\n}\n";
  assert.equal(verdict(`\n${located}`, located), "position-sensitive");
  assert.deepEqual(compareRust("a.rs", rustBase, rustBase.replace("b: bool", "c: bool")).first, {
    before: "2:22 b",
    after: "2:22 c",
  });
});

test("Rust token identity lets imports regroup only within their run", () => {
  const imports =
    "//! Docs.\n\nuse crate::a;\nuse std::fmt;\n#[cfg(test)]\nuse vize_l0::b;\n\nfn f() {}\n";
  const grouped =
    "//! Docs.\n\nuse std::fmt;\n\n#[cfg(test)]\nuse vize_l0::b;\n\nuse crate::a;\n\nfn f() {}\n";
  assert.equal(compareRust("a.rs", imports, grouped).verdict, "use-reordered");
  const moved =
    "//! Docs.\n\nuse std::fmt;\n#[cfg(test)]\nuse vize_l0::b;\n\nfn f() {}\nuse crate::a;\n";
  assert.equal(compareRust("a.rs", imports, moved).verdict, "different");
});

test("AST identity ignores prose comments but not semantic ones", () => {
  const before = "// Builds the value (#1234).\nexport const v = /* @__PURE__ */ make(1);\n";
  assert.equal(
    compareScriptFile("a.ts", before, "export const v = /* @__PURE__ */ make(1);\n").verdict,
    "identical",
  );
  assert.equal(
    compareScriptFile("a.ts", before, "// Builds the value.\nexport const v = make(1);\n").verdict,
    "semantic-comment",
  );
  assert.equal(
    compareScriptFile("a.ts", before, before.replace("(1)", "(2)")).verdict,
    "different",
  );
  assert.equal(compareScriptFile("a.ts", before, "export const = ;\n").verdict, "parse-error");
});

test("AST identity compares Vue scripts as trees and the rest as bytes", () => {
  const sfc =
    '<script setup lang="ts">\n// The count (#1234).\nconst count = 1;\n</script>\n\n<template>\n  <!-- kept -->\n  <p>{{ count }}</p>\n</template>\n';
  assert.equal(
    compareScriptFile("a.vue", sfc, sfc.replace("// The count (#1234).\n", "")).verdict,
    "identical",
  );
  assert.equal(
    compareScriptFile("a.vue", sfc, sfc.replace("  <!-- kept -->\n", "")).verdict,
    "template-changed",
  );
});

const module = (line: number, opcode: string, file = "src/lib.rs") => {
  const path = `${file}\\00`;
  const size = file.length + 1;
  const length = `\\${file.length.toString(16).padStart(2, "0").toUpperCase()}`;
  return [
    `@alloc_a = private unnamed_addr constant [${size} x i8] c"${path}", align 1`,
    `@alloc_b = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_a, [16 x i8] c"${length}\\00\\00\\00\\00\\00\\00\\00\\${line.toString(16).padStart(2, "0")}\\00\\00\\00\\05\\00\\00\\00" }>, align 8`,
    "",
    "define void @_RNvCs0_4demo4walk(i64 %a) {",
    "start:",
    `  %x = ${opcode} i64 %a, 1`,
    "  call void @_RNvNtCs1_4core9panicking5panic(ptr @alloc_b)",
    "  ret void",
    "}",
    "",
  ].join("\n");
};

test("IR identity normalizes panic-location lines and columns only", () => {
  assert.equal(normalizeLocations(module(10, "add")).summary.sites, 1);
  const moved = compareWithLocations("demo", module(10, "add"), module(12, "add"));
  assert.deepEqual([moved.changed, moved.identical], [[], 1]);
  assert.deepEqual(moved.locations, {
    base: { sites: 1, files: 1, pathBytes: 11 },
    head: { sites: 1, files: 1, pathBytes: 11 },
  });
  const changed = compareWithLocations("demo", module(10, "add"), module(12, "sub"));
  assert.deepEqual(changed.changed, ["demo::walk"]);
  const split = (files: boolean) =>
    compareWithLocations("demo", module(10, "add"), module(3, "add", "src/lib/walk.rs"), {
      files,
    }).changed;
  assert.deepEqual([split(false), split(true)], [["demo::walk"], []]);
});
