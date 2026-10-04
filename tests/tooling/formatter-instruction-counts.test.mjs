import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { loadRegistry } from "../../tools/benchmarks/scripts/instruction-counts-lib.mjs";
import {
  formatterInstructionSuites,
  levelInstructionSuites,
} from "../../tools/benchmarks/scripts/instruction-counts-suites.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const original = fs.readFileSync(path.join(root, "crates/vize_glyph/benches/formatter.rs"));
const formatterIds = [
  "formatter_sfc_simple",
  "formatter_sfc_reuse",
  "formatter_script_large",
  "formatter_template_complex",
];

void test("instruction inputs retain all three complete original formatter benchmark bodies", () => {
  assert.equal(original.length, 5222);
  assert.equal(
    createHash("sha256").update(original).digest("hex"),
    "f84876728c734572b018dec9da79917ceadd46e57d9ab7b48ae515ddebcdbea5",
    "original history source S021 must remain byte-identical",
  );
  const declarations = [
    ...original.toString("utf8").matchAll(/const ([A-Z_]+): &str = r#"([\s\S]*?)"#;/g),
  ];
  assert.deepEqual(
    declarations.map((match) => match[1]),
    ["SIMPLE_SFC", "LARGE_SCRIPT", "COMPLEX_TEMPLATE"],
  );
  const copies = ["simple-sfc.txt", "large-script.txt", "complex-template.txt"];
  const sizes = [454, 1083, 1279];
  for (const [index, declaration] of declarations.entries()) {
    const expected = Buffer.from(declaration[2]);
    const actual = fs.readFileSync(
      path.join(root, "crates/vize_glyph/benches/fixtures/formatter", copies[index]),
    );
    assert.equal(actual.length, sizes[index]);
    assert.deepEqual(actual, expected, `${declaration[1]} must retain leading/trailing bytes`);
  }
});

void test("formatter collection is separate from the unchanged twelve-suite level gate", () => {
  assert.deepEqual(formatterInstructionSuites(), [["vize_glyph", "formatter_instructions"]]);
  assert.deepEqual(
    levelInstructionSuites(root).map(([pkg, bench]) => [
      pkg,
      pkg === "vize_l1_to_l2" && bench === "davinci_storage" ? "l1_to_l2_storage" : bench,
    ]),
    [
      ["davinci_harness", "selfcheck"],
      ["vize_armature", "davinci"],
      ["vize_croquis", "davinci"],
      ["vize_atelier_core", "davinci"],
      ["vize_atelier_dom", "davinci"],
      ["vize_atelier_vapor", "davinci"],
      ["vize_atelier_ssr", "davinci"],
      ["vize_l0", "pass_runtime"],
      ["vize_l0", "fact_runtime"],
      ["vize_l1_to_l2", "l1_to_l2_storage"],
      ["vize_patina", "davinci_markup"],
      ["vize_musea", "davinci_art"],
    ],
  );
  const levelRegistry = loadRegistry(path.join(root, "docs/davinci/plan/budgets.toml"));
  const formatterRegistry = loadRegistry(
    path.join(root, "docs/davinci/plan/formatter-instruction-registry.toml"),
  );
  assert.deepEqual([...formatterRegistry], formatterIds);
  for (const id of formatterIds) assert.ok(!levelRegistry.has(id));
  const registryText = fs.readFileSync(
    path.join(root, "docs/davinci/plan/formatter-instruction-registry.toml"),
    "utf8",
  );
  assert.doesNotMatch(registryText, /allocs\s*=|alloc_bytes_peak\s*=|instructions\s*=/);
});

void test("formatter mode refuses missing real measured caps instead of borrowing level ceilings", () => {
  const child = spawnSync(
    process.execPath,
    [
      "tools/benchmarks/scripts/instruction-counts.mjs",
      "--formatter",
      "--verify-budgets",
      "--budgets",
      "does-not-exist-formatter-caps.toml",
    ],
    { cwd: root, encoding: "utf8" },
  );
  assert.equal(child.status, 1);
  assert.match(child.stderr, /does-not-exist-formatter-caps.toml/);
  const foreign = spawnSync(
    process.execPath,
    [
      "tools/benchmarks/scripts/instruction-counts.mjs",
      "--formatter",
      "--verify-budgets",
      "--budgets",
      "docs/davinci/plan/instruction-budgets.toml",
    ],
    { cwd: root, encoding: "utf8" },
  );
  assert.equal(foreign.status, 1);
  assert.match(foreign.stderr, /missing.*formatter_sfc_simple.*unregistered/);
});
