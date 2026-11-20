import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  ENGINES,
  LEDGER_REL,
  STATUSES,
  SUBSTRATES,
  buildLedger,
  renderLedger,
  renderSummary,
} from "../../tools/support/compat/davinci/lib/lint-substrate.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const generator = "tools/support/compat/davinci/lint-substrate.ts";
let ledger: ReturnType<typeof buildLedger> | undefined;
const rows = () => (ledger ??= buildLedger());
const run = (...args: string[]) =>
  spawnSync(process.execPath, [generator, ...args], { cwd: root, encoding: "utf8" });

void test("the committed ledger matches the rule sources", () => {
  const result = run("--check");
  assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
});

void test("an edited or missing ledger copy is reported stale", () => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "lint-substrate-"));
  const copy = path.join(scratch, "ledger.tsv");
  try {
    assert.equal(run("--write", "--ledger", copy).status, 0);
    assert.equal(
      fs.readFileSync(copy, "utf8"),
      fs.readFileSync(path.join(root, LEDGER_REL), "utf8"),
    );
    assert.equal(run("--check", "--ledger", copy).status, 0);
    const text = fs.readFileSync(copy, "utf8");
    fs.writeFileSync(copy, text.replace("\tlegacy-rule\t", "\tnative\t"));
    const stale = run("--check", "--ledger", copy);
    assert.equal(stale.status, 1, "an edited status row was accepted");
    assert.match(stale.stderr, /drifted from the rule sources/u);
    fs.rmSync(copy);
    const missing = run("--check", "--ledger", copy);
    assert.equal(missing.status, 1);
    assert.match(missing.stderr, /does not exist/u);
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
});

void test("every rule has one substrate, known engines and a consistent status", () => {
  const names = new Set(rows().map((row) => row.rule));
  assert.equal(names.size, rows().length, "duplicate rule rows");
  for (const row of rows()) {
    assert.ok(SUBSTRATES.includes(row.substrate), row.rule);
    assert.ok(STATUSES.includes(row.status), row.rule);
    assert.ok(row.engine.length > 0, `${row.rule} has no engine`);
    for (const engine of row.engine) assert.ok(ENGINES.has(engine), `${row.rule}: ${engine}`);
    if (row.legacy.length > 0) assert.equal(row.status, "legacy-rule", row.rule);
    if (row.engine.includes("relief-visitor")) assert.notEqual(row.status, "native", row.rule);
  }
  const byName = new Map(rows().map((row) => [row.rule, row]));
  // Anchors whose classification follows from their source, not a table.
  assert.deepEqual(byName.get("css/no-important"), {
    rule: "css/no-important",
    file: "css/no_important.rs",
    substrate: "L1 css",
    engine: ["style-l1"],
    legacy: [],
    status: "native",
    l1Callback: false,
  });
  assert.equal(byName.get("vue/no-template-shadow")?.substrate, "L2 semantic");
  assert.equal(byName.get("type/await-thenable")?.engine.join("+"), "corsa");
  // Syntax rules finish on the pure L1 engine, not on the L2 facade.
  const accessKey = byName.get("a11y/no-access-key");
  assert.equal(accessKey?.engine.join("+"), "markup-l2");
  assert.equal(accessKey?.l1Callback, true);
  assert.notEqual(accessKey?.status, "native");
});

void test("the summary counts every row once per substrate", () => {
  const summary = renderSummary(rows());
  const total = new RegExp(
    `\\| total +\\|(?: +\\d+ \\|){${STATUSES.length}} +${rows().length} \\|`,
    "u",
  );
  assert.match(summary, total);
  assert.equal(renderLedger(rows()).split("\n").length, rows().length + 3);
});
