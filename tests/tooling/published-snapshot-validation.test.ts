import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { HEADLINE_CHECK_LANES } from "../../tools/benchmarks/scripts/compare-tools-report.mjs";
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
  const lanes = (surfaceId: string) =>
    artifact.surfaces
      .find((s: { id: string }) => s.id === surfaceId)
      .variants.map((v: { id: string }) => v.id);
  assert.deepEqual(lanes("check").sort(), [...HEADLINE_CHECK_LANES].sort());
  for (const id of [
    "vue-tsc",
    "verter-tsc",
    "vize-check-1t",
    "vize-check-max",
    "tsc-vize-mapper",
    "tsc-vue-mapper",
    "tsrs-vize-mapper",
    "tsrs-vue-mapper",
  ])
    assert.ok(lanes("check-small").includes(id), id);
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
    "mapper host version",
    (d) => {
      d.versions.tsrs = null;
    },
  ],
  [
    "Vue mapper version",
    (d) => {
      d.versions.vueContentMapper = null;
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
  type ProcessVariant = {
    cpuRuns: number[];
    cpuMedianMs: number;
    coldMs: number;
    rssRuns: number[];
    rssMedianKb: number;
  };
  const processFailures: [string, (variant: ProcessVariant) => void][] = [
    ["missing CPU samples", (v) => void Reflect.deleteProperty(v, "cpuRuns")],
    ["short CPU samples", (v) => void v.cpuRuns.pop()],
    ["negative CPU sample", (v) => void (v.cpuRuns[0] = -1)],
    ["CPU median", (v) => void (v.cpuMedianMs += 1)],
    ["missing cold run", (v) => void Reflect.deleteProperty(v, "coldMs")],
    ["no-op cold run", (v) => void (v.coldMs = 0)],
    ["missing peak RSS samples", (v) => void Reflect.deleteProperty(v, "rssRuns")],
    ["fractional peak RSS sample", (v) => void (v.rssRuns[0] = 1.5)],
    ["peak RSS median", (v) => void (v.rssMedianKb += 1)],
    ["missing tool setting", (v) => void Reflect.deleteProperty(v, "tool")],
  ];
  for (const [label, corrupt] of processFailures) {
    test(`publication refuses ${id} ${label}`, () => {
      const data = structuredClone(artifact);
      corrupt(data.surfaces.find((s: { id: string }) => s.id === id).variants.at(-1));
      assert.throws(
        () => validatePublishedSnapshot(data),
        /CPU samples|cold run|peak RSS|tool setting/u,
      );
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
