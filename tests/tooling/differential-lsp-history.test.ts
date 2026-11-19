import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  expectedBuildIdentity,
  validateBuildReceipt,
  type BuildReceipt,
} from "../differential/build-receipt.ts";
import {
  HISTORY_LEDGER,
  loadHistoryManifest,
  normalize,
  pickLabels,
  positionOf,
  runHistoryPack,
} from "../differential/lsp-history.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const binary = path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize");
const TERMINAL = new Set(["response-fixture-covered", "no-response-behavior"]);

function ledgerRows() {
  const [header, ...lines] = fs
    .readFileSync(path.join(root, HISTORY_LEDGER), "utf8")
    .split("\n")
    .filter(Boolean);
  assert.equal(header, "commit\tsubject\tstatus\tfixture\tremaining");
  return lines.map((line) => {
    const [commit, subject, status, fixture, remaining] = line.split("\t");
    return { commit, subject, status, fixture, remaining };
  });
}

void test("LSP fix-history sessions and the ledger agree", () => {
  const { manifest } = loadHistoryManifest(root);
  const legacy = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/differential/lsp/manifest.json"), "utf8"),
  ) as { cases: { id: string }[] };
  const known = new Set([...legacy.cases.map((c) => c.id), ...manifest.cases.map((c) => c.id)]);
  const rows = ledgerRows();
  // The denominator is the 255 fix subjects pinned in the ledger (#6883).
  assert.equal(rows.length, 255);
  assert.equal(new Set(rows.map((row) => row.commit)).size, 255);
  const byCommit = new Map(rows.map((row) => [row.commit, row]));
  for (const entry of manifest.cases) {
    const row = byCommit.get(entry.commit);
    assert(row, `${entry.id}: commit is not a ledger row`);
    assert.equal(row.status, "response-fixture-covered", entry.id);
    assert(row.fixture.split(",").includes(entry.id), `${entry.id}: ledger fixture column`);
  }
  for (const row of rows) {
    if (row.status === "response-fixture-covered" && !row.fixture.includes("/manifest.json")) {
      for (const id of row.fixture.split(",")) assert(known.has(id), `${row.commit}: ${id}`);
    }
    if (row.status === "no-response-behavior") {
      assert.equal(row.fixture, "-");
      assert.match(row.remaining, /Covered by: \S/, `${row.commit}: covering evidence`);
    }
    if (TERMINAL.has(row.status)) continue;
    assert(
      [
        "pending-response-audit",
        "source-reviewed-pending-response",
        "external-pack-prepared",
        "selected-response-prepared",
        "regression-held",
      ].includes(row.status),
      `${row.commit}: unknown status ${row.status}`,
    );
  }
});

void test("LSP fix-history helpers resolve markers, labels and host paths", () => {
  assert.deepEqual(positionOf("ab\nc😀d", { find: "d" }), { line: 1, character: 3 });
  assert.deepEqual(positionOf("x x x", { find: "x", nth: 2, delta: 1 }), {
    line: 0,
    character: 5,
  });
  assert.throws(() => positionOf("abc", { find: "z" }), /marker not found/);
  assert.deepEqual(pickLabels([{ label: "a" }, { label: "b" }], ["b"]), {
    count: 2,
    items: [{ label: "b" }],
  });
  assert.deepEqual(pickLabels({ isIncomplete: false, items: [{ label: "a" }] }, ["z"]), {
    isIncomplete: false,
    count: 1,
    items: [],
  });
  assert.deepEqual(normalize({ uri: "file:///tmp/w/App.vue", text: "/tmp/w/x" }, "/tmp/w"), {
    uri: "${workspace}/App.vue",
    text: "${workspacePath}/x",
  });
});

void test("source-built LSP reproduces every pinned fix-history response", async (t) => {
  const receipt = JSON.parse(
    fs.readFileSync(`${binary}.differential-build.json`, "utf8"),
  ) as BuildReceipt;
  validateBuildReceipt(receipt, expectedBuildIdentity(root));
  const results = await runHistoryPack(binary, 6, root);
  const failures = results.filter((result) => !result.equal);
  t.diagnostic(`${results.length - failures.length}/${results.length} sessions equal`);
  if (failures.length > 0) {
    const artifact = path.join(root, "target/differential/lsp-history-failures.json");
    fs.mkdirSync(path.dirname(artifact), { recursive: true });
    fs.writeFileSync(artifact, `${JSON.stringify(failures, null, 2)}\n`);
    t.diagnostic(`mismatches: ${artifact}`);
  }
  assert.deepEqual(
    failures.map((failure) => failure.id),
    [],
    failures.map((failure) => `${failure.id}: ${failure.error ?? ""}`).join("\n"),
  );
});
