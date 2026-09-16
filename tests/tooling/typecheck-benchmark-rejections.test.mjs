import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { measureVariants } from "../../tools/benchmarks/scripts/compare-tools-measure.mjs";
import {
  createSurface,
  ENGINE_CLASSES_BY_SURFACE,
  renderEngineClassSections,
} from "../../tools/benchmarks/scripts/compare-tools-report.mjs";
import { normalizeTypecheckResult } from "../../tools/benchmarks/scripts/typecheck-command.mjs";
import {
  gateTypecheckVariants,
  recordTypecheckRejection,
} from "../../tools/benchmarks/scripts/typecheck-readiness.mjs";

const diagnostic = "Base.vue(1,30): error TS2322: not a number";
const result = { status: 1, stdout: diagnostic, stderr: "", ms: 1 };

void test("unexpected failure output cannot hide behind a real diagnostic", () => {
  for (const message of [
    "Error: crash",
    "TypeError: crash",
    "SyntaxError: crash",
    "Error [ERR_MODULE_NOT_FOUND]: golar",
    "panic: crash",
    "arbitrary output",
    "  TypeError: crash",
    "    at run (cli.js:1:1)",
  ]) {
    for (const stream of ["stdout", "stderr"]) {
      assert.throws(
        () =>
          normalizeTypecheckResult(
            {
              ...result,
              [stream]: `${result[stream]}\n${message}`,
            },
            process.cwd(),
          ),
        /unexpected type-check output/u,
      );
    }
  }
  assert.equal(
    normalizeTypecheckResult(
      {
        ...result,
        stdout: `Using config from ./golar.config.ts...\n${diagnostic}\n  Property 'value' is missing.`,
      },
      process.cwd(),
    ).diagnostics.length,
    1,
  );
  assert.throws(
    () =>
      normalizeTypecheckResult(
        {
          status: 0,
          stdout: JSON.stringify({ files: [], errorCount: 0, warningCount: 0 }),
          stderr: "Error: crash",
        },
        process.cwd(),
        "json",
      ),
    /stderr/u,
  );
  assert.equal(
    normalizeTypecheckResult(
      {
        ...result,
        stderr: "verter-tsc: checking 1 .vue file(s)...\nFound 1 error(s) in 1 file(s).",
      },
      process.cwd(),
    ).diagnostics.length,
    1,
  );
  assert.throws(
    () =>
      normalizeTypecheckResult(
        {
          ...result,
          stderr: "Found 2 error(s) in 1 file(s).",
        },
        process.cwd(),
      ),
    /summary count/u,
  );
});

void test("optional preflight failures remain visible and never become measured lanes", (t) => {
  const dir = mkdtempSync(join(tmpdir(), "typecheck-rejection-"));
  t.after(() => {
    for (const suffix of ["", "-readiness", "-gate-plant"])
      rmSync(`${dir}${suffix}`, { recursive: true, force: true });
  });
  writeFileSync(join(dir, "tsconfig.json"), JSON.stringify({ include: [] }));
  const rejected = [];
  const lane = {
    id: "golar-typecheck",
    label: "Golar typecheck",
    run: () => {
      throw new Error("could not check project");
    },
  };
  assert.deepEqual(
    gateTypecheckVariants(
      [lane],
      dir,
      () => {},
      (...args) => {
        recordTypecheckRejection(rejected, ...args);
      },
    ),
    [],
  );
  assert.deepEqual(rejected, [
    {
      id: lane.id,
      label: lane.label,
      phase: "preflight",
      reason: "could not check project",
    },
  ]);
  assert.throws(
    () => recordTypecheckRejection([], { id: "vize-check-max" }, new Error("broken"), "preflight"),
    /broken/u,
  );
});

void test("warmup and later measurement failures discard every sample of the rejected lane", async () => {
  for (const failAt of [1, 3]) {
    const calls = [];
    const rejected = [];
    let count = 0;
    const variants = [
      {
        id: "golar-default",
        label: "Golar",
        files: 1,
        measure: () => {
          count++;
          if (count === failAt) throw new Error("diagnostics changed");
          return 1;
        },
      },
      {
        id: "vize-check-max",
        label: "Vize",
        files: 1,
        measure: (context) => {
          calls.push(context);
          return 10;
        },
      },
    ];
    const measured = await measureVariants(variants, { warmups: 1, runs: 3 }, (...args) => {
      recordTypecheckRejection(rejected, ...args);
    });
    assert.equal(count, failAt);
    assert.equal(calls.length, 4);
    assert.deepEqual(
      measured.map((v) => v.id),
      ["vize-check-max"],
    );
    assert.deepEqual(measured[0].runs, [10, 10, 10]);
    assert.equal(rejected[0].phase, failAt === 1 ? "warmup" : "measure");
  }
  await assert.rejects(
    measureVariants(
      [
        {
          id: "verter-tsc",
          measure: () => {
            throw new Error("required failed");
          },
        },
      ],
      { warmups: 1, runs: 1 },
      (...args) => recordTypecheckRejection([], ...args),
    ),
    /required failed/u,
  );
});

void test("rejected optional comparators are accounted for but cannot affect a ratio or ranking", () => {
  const rejectedVariants = ["golar-default", "golar-typecheck"].map((id) => ({
    id,
    label: id,
    phase: "preflight",
    reason: "unstable diagnostics",
  }));
  const input = {
    id: "check",
    label: "Type check",
    files: 500,
    baselineId: "vue-tsc",
    vizeSingleId: "vize-check-1t",
    vizeMaxId: "vize-check-max",
    engineClasses: ENGINE_CLASSES_BY_SURFACE.check,
    variants: ["vue-tsc", "verter-tsc", "vize-check-1t", "vize-check-max"].map((id) => ({
      id,
      label: id,
      medianMs: id === "verter-tsc" ? 20 : 10,
    })),
    rejectedVariants,
  };
  const surface = createSurface(input);
  assert.equal(surface.primarySpeedup, 2);
  assert.equal(surface.speedupBaselineId, "verter-tsc");
  assert.ok(
    surface.engineClassRanking.flatMap((g) => g.rows).every((row) => !row.id.startsWith("golar")),
  );
  const rendered = renderEngineClassSections([surface], String).join("\n");
  assert.match(rendered, /golar-default: rejected during preflight; no timing or rank published/u);
  assert.match(rendered, /unstable diagnostics/u);
  for (const id of ["verter-tsc", "vize-check-max", "vue-tsc"]) {
    assert.throws(
      () =>
        createSurface({
          ...input,
          variants: input.variants.filter((v) => v.id !== id),
          rejectedVariants: [...rejectedVariants, { id, phase: "preflight", reason: "failed" }],
        }),
      /invalid rejected/u,
    );
  }
  assert.throws(() => createSurface({ ...input, rejectedVariants: [] }), /missing required/u);
});
