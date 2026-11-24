import assert from "node:assert/strict";
import { test } from "node:test";

import { compareScriptFile } from "../../tools/commands/levels/ast-identity.ts";
import {
  compareWithLocations,
  normalizeLocations,
} from "../../tools/commands/levels/ir-identity-locations.ts";
import { compareRust } from "../../tools/commands/levels/rust-token-identity.ts";
import { lexRust } from "../../tools/support/readability/rust-tokens.ts";

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

test("identity keeps semantic comments with their call, and reads only top-level SFC scripts", () => {
  const left = "const a = /*#__PURE__*/ f(1n); const b = f(2n);";
  assert.equal(compareScriptFile("a.ts", left, "// prose\n" + left).verdict, "identical");
  const moved = "const a = f(1n); const b = /*#__PURE__*/ f(2n);";
  assert.equal(compareScriptFile("a.ts", left, moved).verdict, "semantic-comment");
  const fake = "<template><!-- <script>/* prose */</script> --><p/></template>";
  assert.equal(
    compareScriptFile("a.vue", fake, fake.replace("prose", "edited")).verdict,
    "template-changed",
  );
});

test("only exclusively used panic caller constants are normalized", () => {
  const ir = module(10, "add");
  assert.equal(
    normalizeLocations(ir.replace(/@_RNvNtCs1_4core9panicking5panic/gu, "@store")).summary.sites,
    0,
  );
  assert.equal(
    normalizeLocations(ir + "declare void @keep(ptr)\n@table = global ptr @alloc_b\n").summary
      .sites,
    0,
  );
  const text = "a".repeat(150_000);
  assert.equal(
    normalizeLocations(`@long = private constant [${text.length} x i8] c"${text}"`).summary.sites,
    0,
  );
});

test("Rust punctuation, position macros and conditional bindings keep their distinctions", () => {
  const same = (a: string, b: string) => compareRust("lib.rs", a, b).verdict;
  assert.equal(same("let a = (x,/* note */y);", "let a = (x, y);"), "identical");
  assert.equal(same("// note\nif line != 0 {}", "if line != 0 {}"), "identical");
  assert.equal(same("// note\nlet n = line!();", "let n = line!();"), "position-sensitive");
  const conditional = '#[cfg_attr(feature="cli", derive(clap::Parser))]\n';
  assert.equal(
    same("/// First\n" + conditional + "struct App;", "/// Second\n" + conditional + "struct App;"),
    "doc-product-text",
  );
});

test("lint rule configuration and disable-line targets are semantic", () => {
  const verdict = (a: string, b: string) => compareScriptFile("a.ts", a, b).verdict;
  for (const comment of ['/* eslint no-alert: "off" */', '/* oxlint no-alert: "off" */']) {
    assert.equal(verdict(comment + "\nalert(1);", "alert(1);"), "semantic-comment");
    assert.equal(
      verdict("// prose\n" + comment + "\nalert(1);", comment + "\nalert(1);"),
      "identical",
    );
  }
  for (const newline of ["\n", "\r\n"]) {
    const directive = "// eslint-disable-line no-alert";
    assert.equal(
      verdict("alert(1); /* prose */ " + directive, "alert(1);" + newline + directive),
      "semantic-comment",
    );
    assert.equal(
      verdict("alert(1); " + directive, "alert(1); /* prose */ " + directive),
      "identical",
    );
  }
});

test("panic locations accept attributed pointers and retain equal-length path changes", () => {
  const attributed = (line: number, path: string) =>
    module(line, "add", path).replace(
      "panic(ptr @alloc_b)",
      "panic(ptr noalias nofree noundef readonly align 8 captures(none) dereferenceable(24) @alloc_b)",
    );
  const a = attributed(10, "src/lib.rs");
  const b = attributed(24, "src/mod.rs");
  assert.equal(normalizeLocations(a).summary.sites, 1);
  assert.deepEqual(compareWithLocations("demo", a, attributed(24, "src/lib.rs")).changed, []);
  assert.deepEqual(compareWithLocations("demo", a, b).changed, ["demo::walk"]);
  assert.deepEqual(compareWithLocations("demo", a, b, { files: true }).changed, []);
  assert.equal(normalizeLocations(a + "@other = global ptr @alloc_b\n").summary.sites, 0);
});

test("Rust delimiters and suffixes retain their token-tree semantics", () => {
  const verdict = (a: string, b: string) => compareRust("lib.rs", a, b).verdict;
  assert.equal(verdict("fn f() {}", "fn f(){}"), "identical");
  assert.equal(verdict("m!(x && y);", "m!(x & & y);"), "different");
  for (const literal of ['"x"', 'br##"x"##', "b'v'", "'q'", 'c"str"']) {
    assert.equal(verdict("m!(" + literal + "tag);", "m!(" + literal + " tag);"), "different");
  }
});

test("import sorting leaves token-tree arguments and macro bodies in source order", () => {
  const verdict = (a: string) =>
    compareRust("lib.rs", a, a.replace("use a; use b;", "use b; use a;")).verdict;
  for (const code of [
    "m! {use a; use b;}",
    "foo::m!([ { use a; use b; } ]);",
    "macro_rules! m { () => { use a; use b; } }",
    "macro m() { use a; use b; }",
    "#![m({ use a; use b; })] fn f() {}",
  ])
    assert.equal(verdict(code), "different");
  assert.equal(verdict("mod real { use a; use b; }"), "use-reordered");
});

test("moving calls to local wrappers of position macros changes their result", () => {
  for (const macro of ["line", "column"]) {
    const definitions =
      "macro_rules! outer { () => { here!() } }\n" +
      "macro_rules! here { () => { " +
      macro +
      "!() } }\n";
    const call = "const N: u32 = outer!();";
    assert.equal(
      compareRust("a.rs", definitions + "// prose\n" + call, definitions + call).verdict,
      "position-sensitive",
    );
  }
});

test("Rust column macros count Unicode scalars and token slices keep UTF-16 offsets", () => {
  for (const name of ["here", "𐐀"]) {
    const source = (comment: string) =>
      `macro_rules! ${name} { () => { column!() } }\nconst C: u32 = /*${comment}*/ ${name}!();`;
    const before = source("aa");
    const after = source("😀");
    assert.equal(compareRust("a.rs", before, after).verdict, "position-sensitive");
    assert.equal(compareRust("a.rs", before, source("bb")).verdict, "identical");
    const call = lexRust(after).tokens.find((token) => token.line === 2 && token.text === name);
    assert.ok(call);
    assert.equal(call.column, 22);
    assert.equal(call.start, after.lastIndexOf(name));
    assert.equal(after.slice(call.start, call.end), name);
  }
});
