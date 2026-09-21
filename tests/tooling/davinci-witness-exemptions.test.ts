import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  deriveInventory,
  formatInventory,
  inventoryRelative,
  parseInventory,
  ratchetViolations,
  readBaseInventory,
  resolveBaseRef,
  type ExemptionRow,
} from "./support/davinci-witness-exemptions.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const committed = fs.readFileSync(path.join(repoRoot, inventoryRelative), "utf8");

const row = (producer: string, code: string, exempt: number): ExemptionRow => ({
  producer,
  code,
  exempt,
});

function write(root: string, relative: string, source: string): void {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), source);
}

const commit = [
  "-c",
  "user.name=Vize",
  "-c",
  "user.email=vize@example.com",
  "-c",
  "commit.gpgsign=false",
  "commit",
  "-q",
];

function git(cwd: string, args: string[]): string {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  return result.stdout.trim();
}

test("the committed inventory is exactly the one derived from the sources", () => {
  const derived = deriveInventory(repoRoot);
  assert.deepEqual(derived.issues, []);
  assert.equal(committed, formatInventory(derived.rows));
  assert.deepEqual(parseInventory(committed), derived.rows);
});

test("the inventory only shrinks against the base revision", () => {
  const base = resolveBaseRef(repoRoot, process.env);
  const head = parseInventory(committed);
  assert.deepEqual(ratchetViolations(readBaseInventory(repoRoot, base), head), []);
});

test("an injected increase, and a new code for a known producer, are rejected", () => {
  const base = [row("vize_s1_to_s2", "lowering", 2), row("vize_s1_to_s2", "v-slot", 1)];
  const head = [
    row("vize_patina", "vue/no-dupe-keys", 1),
    row("vize_s1_to_s2", "lowering", 3),
    row("vize_s1_to_s2", "surface-syntax", 1),
  ];
  assert.deepEqual(ratchetViolations(base, head), [
    "vize_s1_to_s2/lowering: exempt rose from 2 to 3",
    "vize_s1_to_s2/surface-syntax: new exemption for a producer already on the channel",
  ]);
  const drained = [row("vize_s1_to_s2", "lowering", 1)];
  assert.deepEqual(ratchetViolations(base, drained), []);
  assert.deepEqual(ratchetViolations(undefined, head), []);
});

test("an injected increase fails against a real base revision", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-witness-exemptions-"));
  git(cwd, ["init", "-q"]);
  assert.equal(git(cwd, [...commit, "--allow-empty", "-m", "before the inventory"]), "");
  const introducedAt = git(cwd, ["rev-parse", "HEAD"]);
  write(cwd, inventoryRelative, formatInventory([row("vize_s1_to_s2", "lowering", 1)]));
  git(cwd, ["add", inventoryRelative]);
  git(cwd, [...commit, "-m", "inventory"]);
  const base = git(cwd, ["rev-parse", "HEAD"]);
  const increased = parseInventory(formatInventory([row("vize_s1_to_s2", "lowering", 2)]));
  assert.equal(readBaseInventory(cwd, introducedAt), undefined);
  assert.deepEqual(readBaseInventory(cwd, base), [row("vize_s1_to_s2", "lowering", 1)]);
  assert.deepEqual(ratchetViolations(readBaseInventory(cwd, base), increased), [
    "vize_s1_to_s2/lowering: exempt rose from 1 to 2",
  ]);
  assert.equal(resolveBaseRef(cwd, { WITNESS_EXEMPTIONS_BASE_REF: base }), base);
  assert.throws(() => readBaseInventory(cwd, "no-such-ref"), {
    message: "base revision no-such-ref is not available",
  });
});

test("exemptions are derived from named statics and counted by construction site", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-witness-derive-"));
  write(
    root,
    "crates/producer_a/src/lib.rs",
    [
      '//! static DOC: Exemption = Exemption::new("producer_a", "doc");',
      'pub static KIND: Exemption = Exemption::new("producer_a", "kind");',
      'static UNUSED: Exemption = Exemption::new("producer_a", "unused");',
      'static FORGED: Exemption = Exemption::new("producer_b", "forged");',
      "fn a() { Diagnostic::legacy_error(&KIND, Stage::Surface, span, m); }",
      "fn b() { Diagnostic::legacy_error(&crate::KIND, Stage::Surface, span, m); }",
      "// Diagnostic::legacy_error(&KIND, ..) in a comment is not a site",
      'fn c() { let inline = Exemption::new("producer_a", "inline"); }',
      "",
    ].join("\n"),
  );
  write(root, "crates/producer_a/tests/t.rs", "fn t() { let _ = &KIND; }\n");
  write(root, "crates/producer_c/src/lib.rs", "pub fn nothing() {}\n");
  assert.deepEqual(deriveInventory(root, []), {
    rows: [row("producer_a", "kind", 2)],
    issues: [
      "crates/producer_a/src/lib.rs:4: producer producer_b must be its crate producer_a",
      "crates/producer_a/src/lib.rs:8: an Exemption must be declared as one named static",
      "crates/producer_a/src/lib.rs:3: UNUSED is declared but unused",
    ],
  });
});

test("a contract table contributes one row per error rule", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-witness-table-"));
  const table = {
    producer: "producer_t",
    constructor: "crates/producer_t/src/contracts.rs",
    rows: "crates/producer_t/src/contracts/table.rs",
  };
  write(
    root,
    table.constructor,
    "const fn entry(name: &'static str) -> Entry {\n    Entry { exemption: Exemption::new(PRODUCER, name) }\n}\n",
  );
  write(
    root,
    table.rows,
    [
      '    row!("vue/a", Exact, DIRECTIVES, Error),',
      '    row!("vue/b", Heuristic, STYLE, Warning),',
      '    row!("vue/c", Complete, BINDINGS, Error),',
      '    row!("vue/d" , Exact, STYLE, Error),',
      "",
    ].join("\n"),
  );
  assert.deepEqual(deriveInventory(root, [table]), {
    rows: [row("producer_t", "vue/a", 1), row("producer_t", "vue/c", 1)],
    issues: ["crates/producer_t/src/contracts/table.rs:4: a table row must be one row!(..) line"],
  });
});

test("the inventory parser accepts only the canonical form", () => {
  const cases: Array<[string, string]> = [
    ["producer\tcode\n", "the inventory header drifted"],
    ["producer\tcode\texempt\nvize_x\tk\t1", "the inventory must end with one newline"],
    ["producer\tcode\texempt\nvize_x\tk\n", "line 2: expected 3 fields"],
    ["producer\tcode\texempt\nVize\tk\t1\n", "line 2: names must be [a-z0-9_./-]"],
    ["producer\tcode\texempt\nvize_x\tk\t0\n", "line 2: exempt must be a positive count"],
    [
      "producer\tcode\texempt\nvize_x\tk\t1\nvize_x\tk\t2\n",
      "line 3: rows must be sorted and unique",
    ],
    [
      "producer\tcode\texempt\nvize_y\tk\t1\nvize_x\tk\t2\n",
      "line 3: rows must be sorted and unique",
    ],
  ];
  for (const [source, message] of cases) {
    assert.throws(() => parseInventory(source), { message }, JSON.stringify(source));
  }
});
