// The htmlish divergence ledger (docs/davinci/plan/htmlish-divergences.json) is
// machine-checked: every row states both dialects and points at real fixtures
// (or is owed), every profile switch it names exists, and every deliberate
// divergence from an oracle names its decision, fixture and fuzz exemption.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { packetPath, packets } from "../../tools/commands/levels/htmlish-oracle.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (file: string) => fs.readFileSync(path.join(repoRoot, file), "utf8");

interface Ledger {
  schema: string;
  dialects: Record<string, { files: string; profile: string; oracle: string; norm: string }>;
  divergences: {
    id: string;
    decidedBy: string;
    summary: string;
    fixture: string;
    fuzzExempt: string;
  }[];
  rows: {
    id: string;
    row: string;
    switch: string | null;
    document: string;
    vue: string;
    fixtures: Record<string, string | null>;
  }[];
}

const ledger = JSON.parse(read("docs/davinci/plan/htmlish-divergences.json")) as Ledger;
const owed = (
  JSON.parse(read("docs/davinci/plan/input-placement-owed.json")) as { entries: string[] }
).entries;

test("every dialect names its profile, oracle and norm", () => {
  assert.equal(ledger.schema, "vize.htmlish-divergences");
  assert.deepEqual(Object.keys(ledger.dialects).sort(), ["document", "vue"]);
  for (const dialect of Object.values(ledger.dialects))
    for (const field of ["files", "profile", "oracle", "norm"] as const)
      assert.ok(dialect[field].length > 0, field);
});

test("rows are unique, state both dialects and name real profile switches", () => {
  const profile = read("levels/vize_l1/src/markup/profile.rs");
  const ids = ledger.rows.map((row) => row.id);
  assert.equal(new Set(ids).size, ids.length, "duplicate row id");
  for (const row of ledger.rows) {
    assert.ok(row.row.length > 0 && row.document.length > 0 && row.vue.length > 0, row.id);
    assert.deepEqual(Object.keys(row.fixtures).sort(), Object.keys(ledger.dialects).sort(), row.id);
    if (row.switch !== null)
      assert.match(
        profile,
        new RegExp(`const ${row.switch}: bool`, "u"),
        `${row.id}: ${row.switch}`,
      );
  }
});

test("fixtures exist, and exactly the missing ones are owed", () => {
  const missing: string[] = [];
  for (const row of ledger.rows)
    for (const [dialect, fixture] of Object.entries(row.fixtures)) {
      if (fixture === null) missing.push(`fixture:${row.id}:${dialect}`);
      else
        assert.ok(fs.existsSync(path.join(repoRoot, fixture)), `${row.id}/${dialect}: ${fixture}`);
    }
  assert.deepEqual(
    owed.filter((entry) => entry.startsWith("fixture:")).sort(),
    missing.sort(),
    "the owed list's fixture entries must match the ledger's missing fixtures",
  );
});

test("every deliberate divergence names its decision, fixture and fuzz exemption", () => {
  assert.ok(ledger.divergences.length > 0);
  for (const divergence of ledger.divergences) {
    const [file, anchor] = divergence.decidedBy.split("#");
    const record = read(file);
    assert.ok(anchor, `${divergence.id}: decidedBy needs a section anchor`);
    const headings = [...record.matchAll(/^#+\s+(.+)$/gmu)].map((match) =>
      match[1]
        .toLowerCase()
        .replace(/[`]/gu, "")
        .replace(/[^\w\s-]/gu, "")
        .trim()
        .replace(/\s+/gu, "-"),
    );
    assert.ok(headings.includes(anchor), `${divergence.id}: no section #${anchor} in ${file}`);
    assert.ok(fs.existsSync(path.join(repoRoot, divergence.fixture)), divergence.fixture);
    assert.ok(divergence.summary.length > 0 && divergence.fuzzExempt.length > 0, divergence.id);
  }
});

test("every oracle packet is the tree its oracle builds today", () => {
  for (const packet of packets())
    assert.equal(
      fs.readFileSync(packetPath(packet), "utf8"),
      JSON.stringify(packet, null, 2) + "\n",
      `${packet.row}.${packet.dialect}: run node tools/commands/levels/htmlish-oracle.ts`,
    );
});
