import assert from "node:assert/strict";
import { test } from "node:test";

import {
  createSurface,
  rankWithinEngineClasses,
} from "../../tools/benchmarks/scripts/compare-tools-report.mjs";
import {
  CHECK_ENGINE_CLASSES,
  CHECK_VARIANTS,
  checkSurfaceInput,
} from "./support/benchmark/check-surface.ts";

test("a cross-engine surface publishes the ratio against its declared incumbent", () => {
  const surface = createSurface(checkSurfaceInput());

  assert.deepEqual(surface, {
    id: "check",
    label: "Type check",
    files: 500,
    bytes: 1_000_000,
    baselineId: "vue-tsc",
    vizeSingleId: "vize-check-1t",
    vizeMaxId: "vize-check-max",
    engineClasses: CHECK_ENGINE_CLASSES,
    variants: CHECK_VARIANTS,
    // vue-tsc, the checker readers run: 8000ms / 500ms. It is a different
    // engine from the Vize lane, which the status and the note disclose.
    primarySpeedup: 16,
    speedupBaselineId: "vue-tsc",
    speedupStatus: "cross-engine",
    engineClassRanking: [
      {
        engineClass: "typescript-js",
        label: "JS TypeScript engine (tsc)",
        rows: [{ id: "vue-tsc", label: "vue-tsc", medianMs: 8000, relativeToFastest: 1 }],
      },
      {
        engineClass: "tsgo-native",
        label: "native TypeScript engine (tsgo)",
        rows: [
          {
            id: "vize-check-max",
            label: "Vize check (max)",
            medianMs: 500,
            relativeToFastest: 1,
          },
          {
            id: "verter-tsc",
            label: "verter-tsc",
            medianMs: 1000,
            relativeToFastest: 2,
          },
          {
            id: "golar-typecheck",
            label: "Golar typecheck",
            medianMs: 1500,
            relativeToFastest: 3,
          },
          {
            id: "vize-check-1t",
            label: "Vize check (1T)",
            medianMs: 2000,
            relativeToFastest: 4,
          },
          {
            id: "golar-default",
            label: "Golar (lint+check)",
            medianMs: 2500,
            relativeToFastest: 5,
          },
        ],
      },
    ],
  });
});

// Dropping the same-engine rows costs the in-class ranking, not the headline:
// the published ratio has always been the declared incumbent's.
test("a cross-engine surface without same-engine incumbents keeps its ratio", () => {
  const nativeIncumbents = ["verter-tsc", "golar-typecheck", "golar-default"];
  const surface = createSurface(
    checkSurfaceInput({
      id: "custom-check",
      variants: CHECK_VARIANTS.filter((variant) => !nativeIncumbents.includes(variant.id)),
    }),
  );

  assert.equal(surface.primarySpeedup, 16);
  assert.equal(surface.speedupBaselineId, "vue-tsc");
  assert.equal(surface.speedupStatus, "cross-engine");
});

test("a cross-engine surface whose declared incumbent did not run publishes nothing", () => {
  const surface = createSurface(
    checkSurfaceInput({
      id: "custom-check",
      variants: CHECK_VARIANTS.filter((variant) => variant.id !== "vue-tsc"),
    }),
  );

  assert.equal(surface.primarySpeedup, null);
  assert.equal(surface.speedupBaselineId, null);
  assert.equal(surface.speedupStatus, "unavailable");
});

test("a same-engine surface keeps its ranked primary speedup", () => {
  const surface = createSurface({
    id: "fmt",
    label: "Format",
    files: 500,
    bytes: 1_000_000,
    baselineId: "prettier-cli",
    vizeSingleId: "vize-fmt-1t",
    vizeMaxId: "vize-fmt-max",
    variants: [
      { id: "prettier-cli", label: "prettier", medianMs: 4000, throughput: "n/a", runs: [4000] },
      { id: "vize-fmt-1t", label: "Vize fmt (1T)", medianMs: 800, throughput: "n/a", runs: [800] },
      {
        id: "vize-fmt-max",
        label: "Vize fmt (max)",
        medianMs: 200,
        throughput: "n/a",
        runs: [200],
      },
    ],
  });

  assert.equal(surface.primarySpeedup, 20);
  assert.equal(surface.speedupBaselineId, "prettier-cli");
  assert.equal(surface.speedupStatus, "ranked");
  assert.equal(surface.engineClassRanking, null);
});

test("a surface without a measurable Vize lane reports no speedup at all", () => {
  const surface = createSurface({
    id: "custom-check",
    label: "Type check",
    files: 1,
    bytes: 1,
    baselineId: "vue-tsc",
    vizeSingleId: null,
    vizeMaxId: "vize-check-max",
    variants: [
      { id: "vue-tsc", label: "vue-tsc", medianMs: 10, throughput: "n/a", runs: [10] },
      {
        id: "vize-check-max",
        label: "Vize check (max)",
        medianMs: 0,
        throughput: "n/a",
        runs: [0],
      },
    ],
  });

  assert.equal(surface.primarySpeedup, null);
  assert.equal(surface.speedupStatus, "unavailable");
});

test("every variant of a class-declaring surface must carry an engine class", () => {
  assert.throws(
    () =>
      rankWithinEngineClasses({
        id: "check",
        engineClasses: { "vue-tsc": "typescript-js" },
        variants: CHECK_VARIANTS,
      }),
    {
      name: "Error",
      message: "compare-tools: variant verter-tsc of surface check has no engine class",
    },
  );
});
