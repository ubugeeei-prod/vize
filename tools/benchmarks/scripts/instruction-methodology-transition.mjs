import { MEASUREMENT_RUST, measurementToolchain } from "./measurement-toolchain.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { isDeepStrictEqual } from "node:util";

const FROM = "rustc 1.98.0 (88d9e12ae 2026-08-18)";
const TO = "rustc 1.99.0 (b940084d7 2026-09-28)";
const RUST1991 = "rustc 1.99.1 (58cf8f9c3 2026-10-13)";
const CURRENT = "rustc 1.100.0 (0ef1bc386 2026-11-09)";
const SOURCE = "1a3996963af208c13cd732420bdee586df88bf7a";
const RUN = "https://github.com/ubugeeei-prod/vize/actions/runs/37653130635";
// Single-toolchain re-baselines, keyed by the base compiler: the same rows and
// inputs measured on the next pinned compiler, citing that measurement.
const REBASES = new Map([
  [
    TO,
    {
      version: "1.99.0",
      source: SOURCE,
      run: RUN,
      rustc: RUST1991,
      rebaseSource: "e40813b3306bf85e5f61d96b19ebd432cae10334",
      rebaseRun: "https://github.com/ubugeeei-prod/vize/actions/runs/39370420052",
    },
  ],
  [
    RUST1991,
    {
      version: "1.99.1",
      source: "e40813b3306bf85e5f61d96b19ebd432cae10334",
      run: "https://github.com/ubugeeei-prod/vize/actions/runs/39370420052",
      rustc: CURRENT,
      rebaseSource: "REBASE_SOURCE_1100",
      rebaseRun: "REBASE_RUN_1100",
    },
  ],
]);
const ORIGINALS = [
  {
    source: "b79010ff63b21612ab1dcc927a221f4000a65fe5",
    run: "https://github.com/ubugeeei-prod/vize/actions/runs/36307058591",
    rows: 100,
    sha256: "cd936164c064d65ab0096c9a11a08dbf0fec24f4e8d45fc315f500d8a71d3aa4",
  },
  {
    source: "c225f11707b6e42c08ea5379df255fb3ddb5e5cd",
    run: "https://github.com/ubugeeei-prod/vize/actions/runs/37163400887",
    rows: 4,
    sha256: "e0814d315b5019daa2a769f29e17d2b559fb6b5acf058aa44b83c8bec09e1897",
  },
];

function canonical(value) {
  if (!value || typeof value !== "object") return value;
  if (Array.isArray(value)) return value.map(canonical);
  return Object.fromEntries(
    Object.keys(value)
      .sort()
      .map((key) => [key, canonical(value[key])]),
  );
}

function identities(budgets) {
  return Object.fromEntries(
    Object.entries(budgets.instruction).map(([id, { instructions: _, ...row }]) => [id, row]),
  );
}

// A re-baseline keeps every row, input and non-rustc method field; caps still
// pass the caller's downward ratchet against the authenticated base registry.
function rebaselineInstructionToolchain(current, base, step) {
  const provenance = `unauthenticated ${step.version} instruction provenance`;
  assert.equal(base.source_commit, step.source, provenance);
  assert.equal(base.recorded_run, step.run, provenance);
  assert.equal(current.methodology.rustc, step.rustc, "unauthorized forward instruction compiler");
  assert.deepEqual(
    { ...current, instruction: identities(current) },
    {
      ...base,
      source_commit: step.rebaseSource,
      recorded_run: step.rebaseRun,
      methodology: { ...base.methodology, rustc: step.rustc },
      instruction: identities(base),
    },
    "toolchain re-baseline must preserve every row, input identity and method field",
  );
}

// Reviewed toolchain transitions. The complete 1.98 contract includes every
// row, cap, input path/digest/window, provenance and non-rustc method field.
export function ratchetInstructionMethodology(current, base) {
  if (isDeepStrictEqual(current.methodology, base.methodology)) return;
  const rebase = REBASES.get(base.methodology.rustc);
  if (rebase) return rebaselineInstructionToolchain(current, base, rebase);
  assert.equal(base.methodology.rustc, FROM, "unauthorized original instruction compiler");
  assert.equal(current.methodology.rustc, TO, "unauthorized forward instruction compiler");
  const original = ORIGINALS.find(
    (item) => item.source === base.source_commit && item.run === base.recorded_run,
  );
  assert.ok(original, "unauthenticated original instruction provenance");
  assert.equal(Object.keys(base.instruction).length, original.rows, "original inventory changed");
  assert.equal(
    createHash("sha256")
      .update(JSON.stringify(canonical(base)))
      .digest("hex"),
    original.sha256,
    "complete original instruction contract changed",
  );
  assert.deepEqual(
    current,
    {
      ...base,
      source_commit: SOURCE,
      recorded_run: RUN,
      methodology: { ...base.methodology, rustc: TO },
    },
    "forward instruction transition must preserve every original cap and identity",
  );
}

export function authenticateInstructionMeasurementSource(report, context) {
  assert.match(context.sourceCommit, /^[a-f0-9]{40}$/, "invalid current checkout identity");
  assert.equal(
    context.toolchain,
    MEASUREMENT_RUST,
    `instruction source toolchain must be ${MEASUREMENT_RUST}`,
  );
  assert.equal(report.methodology.rustc, CURRENT, "instruction measurement compiler mismatch");
  assert.equal(
    report.source_commit,
    context.sourceCommit,
    "measurement source is not current checkout",
  );
}

export function verifyRepositoryInstructionBudgets(budgets, root) {
  measurementToolchain(root);
  assert.equal(budgets.methodology.rustc, CURRENT, "instruction budget compiler mismatch");
}

export function verifyRepositoryInstructionSource(report, root) {
  verifyRepositoryInstructionBudgets(report, root);
  const head = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(head.status, 0, "cannot authenticate current instruction checkout");
  authenticateInstructionMeasurementSource(report, {
    sourceCommit: head.stdout.trim(),
    toolchain: MEASUREMENT_RUST,
  });
}
