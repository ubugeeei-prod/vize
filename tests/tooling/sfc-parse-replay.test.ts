import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";
import * as contract from "../../tools/benchmarks/scripts/sfc-parse-replay-contract.ts";
import * as reporter from "../../tools/benchmarks/scripts/sfc-parse-replay-summary.ts";
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
} from "../../tools/benchmarks/scripts/sfc-parse-replay-contract.ts";
import {
  pairedInterval,
  summarizeRunner,
} from "../../tools/benchmarks/scripts/sfc-parse-replay-summary.ts";

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
  const replay = readFileSync(
    new URL("../../.github/workflows/sfc-parse-replay.yml", import.meta.url),
    "utf8",
  );
  assert.ok(
    replay.indexOf("uses: voidzero-dev/setup-vp@") <
      replay.indexOf("- name: Validate frozen historical pair"),
  );
  assert.match(replay, /node-version-file: harness\/package\.json/u);
  assert.match(replay, /sfc-parse-replay-contract\.ts/u);
});

type Operation = () => unknown;

function outcome(operation: Operation) {
  try {
    return { value: operation() };
  } catch (error) {
    assert.ok(error instanceof Error);
    return { error: { name: error.name, message: error.message } };
  }
}

test("strict replay helpers preserve the authenticated JavaScript outcomes and CLI files", async () => {
  const directory = realpathSync(mkdtempSync(join(tmpdir(), "sfc-replay-types-")));
  try {
    const originals = JSON.parse(
      readFileSync(new URL("./fixtures/sfc-parse-replay-originals.json", import.meta.url), "utf8"),
    ) as { sourceCommit: string; modules: { path: string; sha256: string; base64: string }[] };
    assert.equal(originals.sourceCommit, "a193c99c824b54fbe7e900bb88a19f549cd25ea7");
    assert.deepEqual(
      originals.modules.map(({ path, sha256 }) => ({ path, sha256 })),
      [
        {
          path: "tools/benchmarks/scripts/sfc-parse-replay-contract.mjs",
          sha256: "2329a735c7fd72e849d4f470ba8367ee605a26ca25de53d832fdf213d27e2fb1",
        },
        {
          path: "tools/benchmarks/scripts/sfc-parse-replay-summary.mjs",
          sha256: "3e02fb02f20376e1d7a3d78b10b06aa13c206ca108efc0b912bfea1027394e12",
        },
      ],
    );
    for (const module of originals.modules) {
      const bytes = Buffer.from(module.base64, "base64");
      assert.equal(createHash("sha256").update(bytes).digest("hex"), module.sha256);
      writeFileSync(join(directory, basename(module.path)), bytes);
    }
    const before = (await import(
      pathToFileURL(join(directory, "sfc-parse-replay-contract.mjs")).href
    )) as typeof contract;
    const oldReporter = (await import(
      pathToFileURL(join(directory, "sfc-parse-replay-summary.mjs")).href
    )) as typeof reporter;
    assert.deepEqual(Object.keys(contract), Object.keys(before));
    assert.deepEqual(Object.keys(reporter), Object.keys(oldReporter));
    const compare = (oldOperation: Operation, newOperation: Operation) =>
      assert.deepEqual(outcome(newOperation), outcome(oldOperation));

    for (const base of [BASE_SHA, HEAD_SHA, "", undefined, 1]) {
      for (const conflictingModes of [false, true, "false", undefined]) {
        const input = { base, head: HEAD_SHA, conflictingModes };
        compare(
          () => before.validateRevisionInputs(input),
          () => contract.validateRevisionInputs(input),
        );
      }
    }
    const profile =
      '[profile.release]\nopt-level = 3\nlto = "fat"\ncodegen-units = 1\npanic = "abort"\ndebug = false\ndebug-assertions = false\noverflow-checks = false\nincremental = false\n';
    for (const input of [
      profile,
      "",
      ...profile
        .trimEnd()
        .split("\n")
        .map((line) => profile.replace(line, "")),
    ]) {
      compare(
        () => before.validateReleaseProfile(input),
        () => contract.validateReleaseProfile(input),
      );
    }
    for (const key of [
      "RUSTUP_TOOLCHAIN",
      "RUSTFLAGS",
      "CARGO_ENCODED_RUSTFLAGS",
      "RUSTC",
      "RUSTC_WRAPPER",
      "RUSTC_WORKSPACE_WRAPPER",
      "CARGO_BUILD_TARGET",
      "CARGO_BUILD_RUSTC",
      "CARGO_BUILD_RUSTFLAGS",
      "CARGO_PROFILE_RELEASE_LTO",
    ]) {
      for (const value of [undefined, "", "changed"]) {
        const input = { RUSTUP_TOOLCHAIN: TOOLCHAIN, [key]: value };
        compare(
          () => before.validateBuildEnvironment(input),
          () => contract.validateBuildEnvironment(input),
        );
      }
    }
    for (const side of ["base", "head", "unknown"]) {
      compare(
        () => before.validateFrozenFiles(side, "changed", "", "", profile),
        () => contract.validateFrozenFiles(side, "changed", "", "", profile),
      );
    }
    for (const bytes of [Buffer.alloc(0), Buffer.from("雪🌸"), Buffer.from([0, 255, 1])]) {
      assert.equal(contract.sha256(bytes), before.sha256(bytes));
    }
    assert.deepEqual(contract.pairPlan(), before.pairPlan());
    for (const target of ["", "/tmp/雪 path", "--profile"])
      assert.deepEqual(contract.buildArgs(target), before.buildArgs(target));
    for (const name of [...CASES, "", "throughput"])
      compare(
        () => before.measureArgs(name),
        () => contract.measureArgs(name),
      );
    const finished = { reason: "build-finished", success: true };
    const artifact = {
      reason: "compiler-artifact",
      target: { name: "sfc_parse", kind: ["bench"] },
      executable: "/isolated/雪",
    };
    const stream = (rows: unknown[]) => rows.map((row) => JSON.stringify(row)).join("\n");
    for (const rows of [
      [],
      [artifact, finished],
      [artifact, artifact, finished],
      [artifact],
      [artifact, { ...finished, success: false }],
      [{ ...artifact, target: undefined }, finished],
      [{ ...artifact, executable: 17 }, finished],
      [{ ...artifact, executable: { path: "truthy" } }, finished],
      [{ ...artifact, executable: false }, finished],
    ]) {
      const input = `build-script stdout\n${stream(rows)}\n`;
      compare(
        () => before.parseBuiltExecutable(input),
        () => contract.parseBuiltExecutable(input),
      );
    }
    compare(
      () => before.parseBuiltExecutable("{malformed"),
      () => contract.parseBuiltExecutable("{malformed"),
    );
    for (const values of [[], [1], [3, 1, 2, 4], [1, 0], [1, NaN], [Infinity]])
      compare(
        () => oldReporter.median(values),
        () => reporter.median(values),
      );
    for (const ratios of [
      [1, 2],
      [0.7, 1.1, 1.2, 0.9, 1.3, 1.05],
    ]) {
      for (const seed of [6189, 0, -1])
        compare(
          () => oldReporter.pairedInterval(ratios, seed),
          () => reporter.pairedInterval(ratios, seed),
        );
    }
    const report = {
      baseSha: BASE_SHA,
      headSha: HEAD_SHA,
      runner: "snow 雪",
      settings: SETTINGS,
      observations: observations(),
    };
    const reordered = [...report.observations];
    [reordered[0], reordered[1]] = [reordered[1], reordered[0]];
    for (const input of [
      report,
      { ...report, baseSha: HEAD_SHA },
      { ...report, settings: { ...SETTINGS, pairs: 1 } },
      { ...report, observations: report.observations.slice(1) },
      { ...report, observations: reordered },
      {
        ...report,
        observations: report.observations.map((row) => ({
          ...row,
          estimates: { median: { point_estimate: 0 } },
        })),
      },
    ]) {
      compare(
        () => oldReporter.summarizeRunner(input),
        () => reporter.summarizeRunner(input),
      );
    }
    assert.equal(
      reporter.renderSummary(reporter.summarizeRunner(report)),
      oldReporter.renderSummary(oldReporter.summarizeRunner(report)),
    );
    const traced = (module: typeof contract) => {
      const trace: string[] = [];
      const input = Object.fromEntries(
        ["base", "head", "conflictingModes"].map((key) => [key, undefined]),
      );
      for (const [key, value] of Object.entries({
        base: BASE_SHA,
        head: HEAD_SHA,
        conflictingModes: false,
      }))
        Object.defineProperty(input, key, {
          get() {
            trace.push(key);
            return value;
          },
        });
      const result = outcome(() =>
        module.validateRevisionInputs(
          input as Parameters<typeof contract.validateRevisionInputs>[0],
        ),
      );
      return { trace, result };
    };
    assert.deepEqual(traced(contract), traced(before));

    const inputPath = join(directory, "report.json");
    const outputPath = join(directory, "summary.json");
    const markdownPath = join(directory, "summary.md");
    const oldCLI = join(directory, "sfc-parse-replay-summary.mjs");
    const newCLI = fileURLToPath(
      new URL("../../tools/benchmarks/scripts/sfc-parse-replay-summary.ts", import.meta.url),
    );
    for (const input of [report, { ...report, observations: [] }]) {
      writeFileSync(inputPath, JSON.stringify(input));
      const run = (cli: string) => {
        writeFileSync(outputPath, "untouched JSON");
        writeFileSync(markdownPath, "untouched Markdown");
        const child = spawnSync(process.execPath, [cli, inputPath, outputPath, markdownPath], {
          encoding: "utf8",
        });
        return {
          status: child.status,
          signal: child.signal,
          error: child.error?.message,
          stdout: child.stdout,
          json: readFileSync(outputPath, "utf8"),
          markdown: readFileSync(markdownPath, "utf8"),
        };
      };
      const old = run(oldCLI);
      assert.deepEqual(run(newCLI), old);
      assert.equal(old.status, input.observations.length ? 0 : 1);
    }
    // Exercise the migrated CLI entry directly, without starting the native benchmark driver.
    writeFileSync(inputPath, JSON.stringify(report));
    execFileSync(process.execPath, [newCLI, inputPath, outputPath, markdownPath]);
    assert.deepEqual(
      JSON.parse(readFileSync(outputPath, "utf8")),
      reporter.summarizeRunner(report),
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
