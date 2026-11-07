import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";

export const baselineModes = [
  "frozenPerRule",
  "frozenBatch",
  "scopedPerRule",
  "scopedSharedBatch",
  "scopedEffectiveBatch",
  "scopedCollected",
];

export function baselineMode(environment = process.env) {
  const mode = environment.VIZE_N8N_BASELINE_MODE ?? "serial";
  assert.ok(
    ["serial", "partitioned"].includes(mode),
    "baseline mode must be serial or partitioned",
  );
  return mode;
}

// Fixed contiguous ranges preserve the original input/mode order in the join.
export function baselinePartitions(length) {
  assert.ok(Number.isSafeInteger(length) && length >= 2);
  const split = Math.ceil(length / 2);
  return [
    { index: 0, start: 0, end: split, controls: false },
    { index: 1, start: split, end: length, controls: true },
  ];
}

export function admittedPartition(inventory, environment = process.env) {
  const raw = environment.VIZE_N8N_BASELINE_PARTITION;
  if (raw === undefined) return null;
  const candidate = JSON.parse(raw);
  const parts = baselinePartitions(inventory.files.length);
  assert.ok([0, 1].includes(candidate.index), "unknown fixed partition");
  const expected = parts[candidate.index];
  assert.deepEqual(candidate, expected, "fixed original input partition only");
  return expected;
}

// The parent rejects missing/duplicate/substituted rows without altering any
// complete cold/warm/options/cache payload retained in the original journals.
export function validatePartitionRows(inventory, partition, rows, rulesFor) {
  const selected = inventory.files.slice(partition.start, partition.end);
  assert.equal(rows.length, selected.length * baselineModes.length);
  const totals = {};
  for (const [ordinal, entry] of selected.entries()) {
    for (const [modeIndex, mode] of baselineModes.entries()) {
      const row = rows[ordinal * baselineModes.length + modeIndex];
      assert.equal(
        row.file,
        entry.file,
        "whole original ordered inputs, no duplicate/missing substitution",
      );
      assert.equal(row.mode, mode, "all six original modes in their original per-input order");
      assert.equal(row.sourceSha256, entry.sha256);
      assert.deepEqual(row.rules, rulesFor(entry.file, mode));
      assert.deepEqual(row.warm, row.cold, "whole warm/cold vectors");
      assert.equal(row.warmCalls, 0);
      assert.ok(Number.isFinite(row.elapsedMs) && row.elapsedMs >= 0);
      assert.ok(row.cache && Array.isArray(row.cold));
      const total = (totals[mode] ??= { coldCalls: 0, warmCalls: 0, elapsedMs: 0 });
      total.coldCalls += row.coldCalls;
      total.warmCalls += row.warmCalls;
      total.elapsedMs += row.elapsedMs;
    }
  }
  return totals;
}

// Fully finish each original backing file in the parent before either child.
// Exclusive create alone would allow a sibling to read a partially written file.
export function seedOriginalSources(inventory, directory, checkDeadline = () => {}) {
  checkDeadline();
  fs.mkdirSync(directory, { recursive: true });
  const unique = new Set();
  for (const entry of inventory.files) {
    checkDeadline();
    const bytes = fs.readFileSync(path.join(inventory.fixture, entry.file));
    const hash = createHash("sha256").update(bytes).digest("hex");
    assert.equal(hash, entry.sha256, "complete licensed source matches the original inventory");
    const target = path.join(directory, hash + ".vue");
    if (fs.existsSync(target))
      assert.deepEqual(fs.readFileSync(target), bytes, "original backing bytes remain exact");
    else fs.writeFileSync(target, bytes, { flag: "wx" });
    unique.add(hash);
  }
  checkDeadline();
  return { files: inventory.files.length, uniqueSources: unique.size };
}
