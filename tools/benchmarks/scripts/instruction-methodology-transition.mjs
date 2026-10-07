import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { isDeepStrictEqual } from "node:util";
import { parseTomlLite } from "../../support/compat/davinci/toml-lite.mjs";

const FROM = "rustc 1.98.0 (88d9e12ae 2026-08-18)";
const TO = "rustc 1.99.0 (b940084d7 2026-09-28)";
const SOURCE = "1a3996963af208c13cd732420bdee586df88bf7a";
const RUN = "https://github.com/ubugeeei-prod/vize/actions/runs/37653130635";
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

// One reviewed toolchain transition. The complete old contract includes every
// row, cap, input path/digest/window, provenance and non-rustc method field.
export function ratchetInstructionMethodology(current, base) {
  if (isDeepStrictEqual(current.methodology, base.methodology)) return;
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
  assert.equal(context.toolchain, "1.99.0", "instruction source toolchain must be 1.99.0");
  assert.equal(report.methodology.rustc, TO, "instruction measurement compiler mismatch");
  assert.equal(
    report.source_commit,
    context.sourceCommit,
    "measurement source is not current checkout",
  );
}

export function verifyRepositoryInstructionBudgets(budgets, root) {
  const toolchain = parseTomlLite(fs.readFileSync(path.join(root, "rust-toolchain.toml"), "utf8"))
    .toolchain.channel;
  assert.equal(toolchain, "1.99.0", "instruction source toolchain must be 1.99.0");
  assert.equal(budgets.methodology.rustc, TO, "instruction budget compiler mismatch");
}

export function verifyRepositoryInstructionSource(report, root) {
  verifyRepositoryInstructionBudgets(report, root);
  const head = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(head.status, 0, "cannot authenticate current instruction checkout");
  authenticateInstructionMeasurementSource(report, {
    sourceCommit: head.stdout.trim(),
    toolchain: "1.99.0",
  });
}
