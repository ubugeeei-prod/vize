import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  validatePublishedSnapshot,
  assertFreshSnapshot,
} from "../../tools/benchmarks/scripts/published-snapshot-validation.mjs";

const artifact = JSON.parse(
  readFileSync(
    new URL("../../tools/benchmarks/results/tool-benchmark-latest.json", import.meta.url),
    "utf8",
  ),
);

test("publication retains all raw measurements, binary hashes and diagnostic proofs", () => {
  const validated = validatePublishedSnapshot(artifact);
  assert.deepEqual(validated, artifact);
  const ids = artifact.surfaces
    .find((s: { id: string }) => s.id === "check")
    .variants.map((v: { id: string }) => v.id);
  for (const id of ["vue-tsc", "verter-tsc", "vize-check-1t", "vize-check-max"])
    assert.ok(ids.includes(id));
});

test("derived ratios are recomputed instead of trusting stale or inflated claims", () => {
  const corrupted = structuredClone(artifact);
  corrupted.surfaces[0].primarySpeedup = 99999;
  assert.deepEqual(validatePublishedSnapshot(corrupted), artifact);
});

const failures: [string, (data: typeof artifact) => void][] = [
  [
    "schema",
    (d) => {
      d.schemaVersion = 0;
    },
  ],
  [
    "commit",
    (d) => {
      d.commit.sha = "unknown";
    },
  ],
  [
    "run",
    (d) => {
      d.commit.runUrl = "https://example.com/run/1";
    },
  ],
  [
    "runner",
    (d) => {
      d.runner.label = "local";
    },
  ],
  [
    "backend",
    (d) => {
      d.backend.ready = false;
    },
  ],
  [
    "version",
    (d) => {
      d.versions.tsgo = null;
    },
  ],
  [
    "hash",
    (d) => {
      d.binaries.vize = null;
    },
  ],
  [
    "warmup",
    (d) => {
      d.settings.warmups = 0;
    },
  ],
  [
    "runs",
    (d) => {
      d.settings.runs = 1;
    },
  ],
  [
    "surface",
    (d) => {
      d.surfaces.pop();
      d.surfaces.pop();
    },
  ],
  [
    "duplicate surface",
    (d) => {
      d.surfaces.push(d.surfaces[0]);
    },
  ],
  [
    "zero files",
    (d) => {
      d.surfaces[0].files = 0;
    },
  ],
  [
    "missing samples",
    (d) => {
      d.surfaces[0].variants[0].runs.pop();
    },
  ],
  [
    "no-op sample",
    (d) => {
      d.surfaces[0].variants[0].runs[0] = 0;
    },
  ],
  [
    "nonfinite sample",
    (d) => {
      d.surfaces[0].variants[0].runs[0] = Infinity;
    },
  ],
  [
    "median",
    (d) => {
      d.surfaces[0].variants[0].medianMs += 1;
    },
  ],
  [
    "duplicate variant",
    (d) => {
      d.surfaces[0].variants.push(d.surfaces[0].variants[0]);
    },
  ],
];
for (const [label, corrupt] of failures) {
  test(`publication refuses invalid ${label}`, () => {
    const data = structuredClone(artifact);
    corrupt(data);
    assert.throws(() => validatePublishedSnapshot(data));
  });
}

for (const id of ["check", "large-check"]) {
  for (const field of [
    "corpusPlant",
    "strictTemplates",
    "minimalPlants",
    "diagnosticFingerprint",
    "corpusBaselineFingerprint",
    "status",
    "diagnosticCount",
  ]) {
    test(`publication requires ${id} ${field} evidence`, () => {
      const data = structuredClone(artifact);
      const surface = data.surfaces.find((s: { id: string }) => s.id === id);
      delete surface.variants[0].correctness[field];
      assert.throws(() => validatePublishedSnapshot(data));
    });
  }
}

test("imports reject old, future and regressing snapshots without expiring committed evidence", () => {
  const measuredAt = Date.parse(artifact.generatedAt);
  assert.doesNotThrow(() => assertFreshSnapshot(artifact, artifact, measuredAt + 1000));
  assert.throws(() => assertFreshSnapshot(artifact, null, measuredAt + 8 * 86400_000));
  assert.throws(() => assertFreshSnapshot(artifact, null, measuredAt - 120_000));
  assert.throws(() =>
    assertFreshSnapshot(
      artifact,
      { generatedAt: new Date(measuredAt + 1).toISOString() },
      measuredAt,
    ),
  );
  assert.doesNotThrow(() => validatePublishedSnapshot(artifact));
});
