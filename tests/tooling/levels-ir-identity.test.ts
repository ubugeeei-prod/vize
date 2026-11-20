// The IR identity check accepts a pure move and catches a one-instruction change.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { relocator } from "../../tools/commands/levels/ir-identity.ts";
import {
  compareModules,
  demangleV0,
  isIdentical,
  normalizeModule,
} from "../../tools/support/levels/ir-normalize.ts";
import { steps } from "../../tools/commands/davinci/level-layout-move.ts";

test("v0 symbols demangle to paths without crate disambiguators", () => {
  assert.equal(demangleV0("_RNvNtCsjFKpBQnPn3V_5probe1a1f"), "probe::a::f");
  assert.equal(demangleV0("_RNvMNtCsjFKpBQnPn3V_5probe1aNtB2_1S3len"), "<probe::a::S>::len");
  assert.equal(demangleV0("_RNvCs1234_5other1g"), "other::g");
  assert.equal(demangleV0("memcpy"), null);
  // Nested closures end in `00` (two empty identifiers) before the crate backref.
  assert.equal(
    demangleV0("_RNCNCNvNtCsSD3gOHEo3e_7vize_l14text13encoded_brace00B7_"),
    "vize_l1::text::encoded_brace::{closure}::{closure}",
  );
});

const vtableModule = (target: string, guid: number) => `
@vtable.${guid % 7} = private unnamed_addr constant <{ ptr }> <{ ptr @${target} }>, align 8, !guid !${guid}
define void @_RNvCsAAAA_5probe3use(ptr %f) unnamed_addr #0 {
start:
  call void @_RNvCsAAAA_5probe4call(ptr @vtable.${guid % 7})
  ret void
}
attributes #0 = { nounwind }
`;

test("anonymous globals compare by contents, not by their numbering", () => {
  const base = normalizeModule(vtableModule("_RNvCsAAAA_5probe1a", 3));
  const renumbered = normalizeModule(vtableModule("_RNvCsBBBB_5probe1a", 12));
  assert.ok(isIdentical(compareModules(base, renumbered)));
  const retargeted = normalizeModule(vtableModule("_RNvCsAAAA_5probe1b", 3));
  assert.deepEqual(compareModules(base, retargeted).changed, ["probe::use"]);
});

test("the CLI compares saved IR and exits 1 on a change", (t) => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ir-identity-cli-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  const write = (name: string, text: string) => {
    const file = path.join(dir, name);
    fs.writeFileSync(file, text);
    return file;
  };
  const add = "  %0 = mul i64 %x, 3";
  const base = write("base.ll", module("_RNvNtCsAAAA_5probe1a1f", add));
  const moved = write("moved.ll", module("_RNvNtCsBBBB_5probe1b1f", add, "alloc_ffff", 9));
  const changed = write("changed.ll", module("_RNvNtCsAAAA_5probe1a1f", "  %0 = mul i64 %x, 5"));
  const cli = new URL("../../tools/commands/levels/ir-identity.ts", import.meta.url);
  const run = (...args: string[]) =>
    spawnSync(process.execPath, [fileURLToPath(cli), ...args], { encoding: "utf8" });
  const same = run("--ir", base, moved, "--rename-path", "probe::a=probe::b");
  assert.equal(same.status, 0, same.stderr);
  assert.match(same.stdout, /^identical/u);
  const different = run("--ir", base, changed, "--diff", "1");
  assert.equal(different.status, 1, different.stderr);
  assert.match(
    different.stdout,
    /changed: probe::a::f\n\s+- .*mul i64 %x, 3\n\s+\+ .*mul i64 %x, 5/u,
  );
});

const module = (name: string, body: string, alloc = "alloc_1a2b", meta = 7) => `
@${alloc} = private unnamed_addr constant [3 x i8] c"abc", align 1
define i64 @${name}(i64 %x) unnamed_addr #0 !guid !${meta} {
start:
${body}
  call void @llvm.assume(i1 true), !noalias !${meta + 1}
  %p = getelementptr i8, ptr @${alloc}, i64 %x
  ret i64 %0
}
attributes #0 = { nounwind uwtable }
`;

test("names, anonymous globals and metadata ids do not matter; instructions do", () => {
  const add = "  %0 = mul i64 %x, 3";
  const base = normalizeModule(module("_RNvNtCsAAAA_5probe1a1f", add));
  // The same function, moved to another module, renumbered globals and metadata.
  const moved = normalizeModule(module("_RNvNtNtCsBBBB_5probe1b1c1f", add, "alloc_ffff", 42));
  const move = relocator({
    steps: [],
    paths: [["probe::a::f", "probe::b::c::f"]],
    identifiers: new Map(),
  });
  assert.ok(
    isIdentical(
      compareModules(normalizeModule(module("_RNvNtCsAAAA_5probe1a1f", add), move), moved),
    ),
  );
  // Without the path rewrite the move is reported, not hidden.
  assert.deepEqual(compareModules(base, moved).onlyBase, ["probe::a::f"]);
  // One more instruction in the same function is a change.
  const changed = normalizeModule(
    module("_RNvNtCsAAAA_5probe1a1f", `${add}\n  %1 = xor i64 %0, %x`),
  );
  assert.deepEqual(compareModules(base, changed).changed, ["probe::a::f"]);
});

test("level-layout-move steps relocate base paths to the head spelling", () => {
  const step = steps["l1-markup"];
  assert.ok(step);
  const move = relocator({ steps: [step], paths: [], identifiers: new Map() });
  assert.equal(
    move("<vize_l1::markup::lex::Lexer>::run"),
    "<vize_l1::markup::htmlish::lex::Lexer>::run",
  );
  assert.equal(move("other::markup::lex::f"), "other::markup::lex::f");
});

const rustc = spawnSync("rustc", ["--version"], { encoding: "utf8" });

test(
  "real rustc IR: a module move is identical, a one-instruction change is not",
  { skip: rustc.status === 0 ? false : "rustc is not on PATH" },
  (t) => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ir-identity-"));
    t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
    const emit = (name: string, source: string) => {
      const file = path.join(dir, `${name}.rs`);
      fs.writeFileSync(file, source);
      const out = path.join(dir, `${name}.ll`);
      execFileSync("rustc", [
        "--edition=2024",
        "--crate-type=lib",
        "--crate-name=probe",
        "-Copt-level=3",
        "-Ccodegen-units=1",
        "--emit=llvm-ir",
        "-o",
        out,
        file,
      ]);
      return fs.readFileSync(out, "utf8");
    };
    const body = (extra: string) =>
      `#[inline(never)] pub fn f(x: u64) -> u64 { x.wrapping_mul(3)${extra} }\n` +
      "pub struct S(pub Vec<u8>);\n" +
      "impl S { #[inline(never)] pub fn len(&self) -> usize { self.0.len() + f(1) as usize } }\n";
    const caller = (at: string) =>
      `pub fn g(v: &[u64]) -> u64 { v.iter().map(|x| ${at}::f(*x)).sum::<u64>() }\n`;
    const base = emit("base", `pub mod a {\n${body("")}}\n${caller("a")}`);
    const moved = emit("moved", `pub mod b { pub mod c {\n${body("")}}}\n${caller("b::c")}`);
    const changed = emit("changed", `pub mod a {\n${body(" ^ x")}}\n${caller("a")}`);
    const move = relocator({
      steps: [],
      paths: [["probe::a", "probe::b::c"]],
      identifiers: new Map(),
    });
    assert.ok(isIdentical(compareModules(normalizeModule(base, move), normalizeModule(moved))));
    const difference = compareModules(normalizeModule(base), normalizeModule(changed));
    assert.deepEqual(difference.changed, ["probe::a::f"]);
    assert.deepEqual([...difference.onlyBase, ...difference.onlyHead], []);
  },
);
