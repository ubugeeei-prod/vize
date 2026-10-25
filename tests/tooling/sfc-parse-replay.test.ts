import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import {
  BASE_SHA,
  CASES,
  HEAD_SHA,
  SETTINGS,
  TOOLCHAIN,
  buildArgs,
  measureArgs,
  parseBuiltExecutable,
  pairPlan,
  validateBuildEnvironment,
  validateFrozenFiles,
  validateReleaseProfile,
  validateRevisionInputs,
} from "../../tools/benchmarks/scripts/sfc-parse-replay-contract.mjs";
import {
  pairedInterval,
  summarizeRunner,
} from "../../tools/benchmarks/scripts/sfc-parse-replay-summary.mjs";

test("historical route rejects current heads, reversed pairs, and mixed routes", () => {
  const valid = { base: BASE_SHA, head: HEAD_SHA, conflictingModes: false };
  validateRevisionInputs(valid);
  for (const override of [
    { base: HEAD_SHA },
    { head: BASE_SHA },
    { head: "a".repeat(40) },
    { conflictingModes: true },
    { conflictingModes: "false" },
  ])
    assert.throws(() => validateRevisionInputs({ ...valid, ...override }));
  assert.throws(
    () => validateFrozenFiles("base", Buffer.from("changed"), Buffer.alloc(0), "", ""),
    /inputs or timed body changed/u,
  );
});

test("source release settings and explicit build compiler cannot be overridden", () => {
  const profile = `[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
debug = false
debug-assertions = false
overflow-checks = false
incremental = false
`;
  validateReleaseProfile(profile);
  assert.throws(() => validateReleaseProfile(profile.replace('lto = "fat"', 'lto = "thin"')));
  assert.throws(() =>
    validateReleaseProfile(profile.replace("codegen-units = 1", "codegen-units = 16")),
  );
  validateBuildEnvironment({ RUSTUP_TOOLCHAIN: TOOLCHAIN });
  for (const override of [
    { RUSTUP_TOOLCHAIN: "stable" },
    { RUSTFLAGS: "-Copt-level=1" },
    { RUSTC: "/different/rustc" },
    { CARGO_PROFILE_RELEASE_LTO: "thin" },
  ])
    assert.throws(() => validateBuildEnvironment({ RUSTUP_TOOLCHAIN: TOOLCHAIN, ...override }));
  const args = buildArgs("/isolated/base");
  assert.equal(args[0], "+1.98.0");
  assert.ok(args.includes("--locked") && args.includes("--no-run"));
  assert.ok(!args.includes("--profile"), "retain the source release benchmark profile");
});

test("every case has three adjacent AB and BA pairs and balanced case positions", () => {
  const plan = pairPlan();
  assert.equal(plan.length, 6);
  for (const name of CASES) {
    assert.deepEqual(
      plan.map((row) => row.cases.indexOf(name)).sort((left, right) => left - right),
      [0, 0, 1, 1, 2, 2],
    );
  }
  assert.equal(plan.filter((row) => row.sides.join("/") === "base/head").length, 3);
  assert.equal(plan.filter((row) => row.sides.join("/") === "head/base").length, 3);
  assert.throws(() => measureArgs("throughput"));
  const args = measureArgs("complex");
  assert.ok(args.includes("--bench"), "Criterion direct execution must measure instead of test");
  assert.equal(args[args.indexOf("--exact") + 1], "sfc_parse/complex");
});

test("verbose build-script stdout cannot hide a failed or ambiguous Cargo artifact", () => {
  const artifact = JSON.stringify({
    reason: "compiler-artifact",
    target: { name: "sfc_parse", kind: ["bench"] },
    executable: "/isolated/sfc_parse",
  });
  const finished = JSON.stringify({ reason: "build-finished", success: true });
  const output = `[quote 1.0.45] cargo:rerun-if-changed=build.rs\n${artifact}\n${finished}\n`;
  assert.equal(parseBuiltExecutable(output), "/isolated/sfc_parse");
  assert.throws(() => parseBuiltExecutable(output.replace('"success":true', '"success":false')));
  assert.throws(() => parseBuiltExecutable(`${artifact}\n${output}`));
  assert.throws(() => parseBuiltExecutable("[quote 1.0.45] cargo:rerun-if-changed=build.rs\n"));
});

function observations() {
  return pairPlan().flatMap(({ pair, cases, sides }) =>
    cases.flatMap((name) =>
      sides.map((side) => ({
        pair,
        name,
        side,
        estimates: { median: { point_estimate: side === "base" ? 100 : 107 } },
      })),
    ),
  );
}

test("uncertainty resamples whole pairs and refuses incomplete or reordered evidence", () => {
  assert.deepEqual(pairedInterval([1.07, 1.07, 1.07, 1.07, 1.07, 1.07]).lower, 1.07);
  assert.throws(
    () => pairedInterval(Array(100).fill(1.07)),
    "Criterion iterations cannot replace six pairs",
  );
  const report = {
    baseSha: BASE_SHA,
    headSha: HEAD_SHA,
    runner: 1,
    settings: SETTINGS,
    observations: observations(),
  };
  const summary = summarizeRunner(report);
  assert.ok(summary.rows.every((row) => row.medianPairedRatio === 1.07));
  assert.throws(() => summarizeRunner({ ...report, observations: report.observations.slice(1) }));
  const reordered = [...report.observations];
  [reordered[0], reordered[1]] = [reordered[1], reordered[0]];
  assert.throws(() => summarizeRunner({ ...report, observations: reordered }), /order drift/u);
});

function assertNormalRoute(workflow: string, job: string, production: boolean) {
  const expression = workflow.match(
    new RegExp(`^  ${job}:\\n    if: \\$\\{\\{ (.+) \\}\\}$`, "mu"),
  )?.[1];
  assert.ok(expression, `missing condition for ${job}`);
  const terms = expression.split(/\s*&&\s*/u);
  const unique = new Set(terms);
  assert.equal(unique.size, terms.length, `${job} repeats a routing guard`);
  for (const required of [
    `${production ? "" : "!"}inputs.production_only`,
    "!inputs.historical_sfc_parse",
    "!inputs.level_memory_only",
    "!inputs.sfc_member_only",
  ]) {
    assert.ok(unique.has(required), `${job} must retain ${required}`);
    unique.delete(required);
  }
  for (const extra of unique) {
    assert.match(extra, /^!inputs\.[a-z_]+$/u, `${job} may only exclude additional modes`);
    const input = extra.slice("!inputs.".length);
    assert.match(workflow, new RegExp(`^      ${input}:$`, "mu"));
  }
}

test("normal workflow exact-head and ancestry guards remain separate from historical routing", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/criterion-bench.yml", import.meta.url),
    "utf8",
  );
  assert.equal(workflow.match(/\[\[ "\$HEAD_SHA" == "\$DISPATCH_SHA" \]\]/gu)?.length, 2);
  assertNormalRoute(workflow, "criterion-ab", false);
  assertNormalRoute(workflow, "production-pairs", true);
  for (const job of ["criterion-ab", "production-pairs"]) {
    for (const mode of ["historical_sfc_parse", "level_memory_only", "sfc_member_only"]) {
      const changed = workflow.replace(
        new RegExp(`(${job}:\\n    if: [^\\n]*)!inputs\\.${mode}`, "u"),
        `$1inputs.${mode}`,
      );
      assert.throws(() => assertNormalRoute(changed, job, job === "production-pairs"));
    }
  }
  assert.match(workflow, /uses: \.\/\.github\/workflows\/sfc-parse-replay.yml/u);
});
