import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  loadProjectionMatrix,
  validateProjectionMatrix,
  verifyProjectionDigest,
  type ProjectionDigest,
} from "./support/davinci-ts40-projection.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("TS-40 current-projection fixture matrix is explicit and non-vacuous", () => {
  const matrix = loadProjectionMatrix(root);
  validateProjectionMatrix(root, matrix);
  assert.ok(matrix.fixtures.length >= 8);
  assert.ok(matrix.unproven.some((item) => item.includes("Davinci or S2")));
});

test("TS-40 verifier fails closed on mapping drift", () => {
  const baseline = digest();
  assert.throws(
    () => verifyProjectionDigest(baseline, { ...baseline, mappingsSha256: "f".repeat(64) }),
    /TS-40 mapping drift/,
  );
});

test("TS-40 verifier fails closed on diagnostic drift", () => {
  const baseline = digest();
  assert.throws(
    () => verifyProjectionDigest(baseline, { ...baseline, diagnosticsSha256: "f".repeat(64) }),
    /TS-40 diagnostic drift/,
  );
});

test("TS-40 baselines are wired into exact Content Mapper CI", () => {
  const workflow = fs.readFileSync(
    path.join(root, ".github/workflows/content-mapper-conformance.yml"),
    "utf8",
  );
  for (const command of [
    "cargo test -p vize --test davinci_ts40_projection_cli -- --nocapture",
    "cargo test -p vize_maestro --test davinci_ts40_projection -- --nocapture",
    "cargo test -p vize_maestro --features legacy --test davinci_ts40_projection -- --nocapture",
    "node --test tests/tooling/davinci-ts40-projection.test.ts",
  ]) {
    assert.ok(workflow.includes(command), `Content Mapper CI is missing ${command}`);
  }
});

function digest(): ProjectionDigest {
  return {
    diagnosticsSha256: "d".repeat(64),
    mappingsSha256: "a".repeat(64),
  };
}
