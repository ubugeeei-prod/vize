#!/usr/bin/env node
/**
 * Release-profile measurement of a base/head source pair: level instruction
 * counts (stages, formatter, SFC parsing) and the shipped `vize` CLI and
 * `vize_vitrine` N-API library sizes, with an exact independent A/A twin per
 * side.
 *
 * Every build of the pair sees the same clock: SOURCE_DATE_EPOCH is the base
 * commit's committer time. libmimalloc-sys compiles mimalloc's banner with
 * __DATE__ and __TIME__, and the linker's string tail merging folds a short
 * literal such as "0" into that time string when its last digit matches. The
 * fold moves every later merged string two bytes, and the address-sensitive
 * string work in the measured windows retires different instruction counts.
 * With the wall clock, about one build in five moved the same 29 rows.
 *
 * The instruction benches build through `[profile.ci-opt]`, overridden to the
 * release codegen (fat LTO, one codegen unit) with Cargo's profile environment
 * variables, so no temporary branch has to edit `Cargo.toml`. The binaries
 * whose bytes ship are built with `--release`.
 *
 * Usage (Actions; see .github/workflows/shipped-profile-measure.yml):
 *   node tools/benchmarks/scripts/release-measure.ts --base <dir> --head <dir> --out <dir> [--no-twins]
 *   node tools/benchmarks/scripts/release-measure.ts --report <out-dir>
 */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  appendFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { glibcFloorTarget } from "../../support/github/glibc-floor.ts";
import {
  publishMovedAttribution,
  retainMeasuredProbes,
  retainSourceCli,
} from "./release-measure-attribution.ts";

/** Instruction suites and the instruction-counts.mjs flags that collect them. */
export const SUITES: ReadonlyArray<readonly [string, readonly string[]]> = [
  ["stages", []],
  ["formatter", ["--formatter"]],
  ["sfc", ["--sfc-parsing"]],
];

export interface InstructionRow {
  instructions: number;
  fixture_sha256: string;
  window: string;
  methodology: Record<string, string>;
}

/** One measured side: per-row instruction measurements and binary sizes. */
export interface Side {
  rows: Record<string, InstructionRow>;
  sizes: Record<string, number>;
  /** SHA-256 of every measured benchmark executable, by `<package>/<bench>`. */
  binaries?: Record<string, string>;
}

/** Fold the three executions of one collect run into one identified measurement per row. */
export function foldRuns(measurement: {
  methodology: Record<string, string>;
  runs: Array<Record<string, { instructions: number; fixture_sha256: string; window: string }>>;
}) {
  const rows: Record<string, InstructionRow> = {};
  for (const run of measurement.runs) {
    for (const [id, row] of Object.entries(run)) {
      const measured = { ...row, methodology: measurement.methodology };
      if (rows[id] !== undefined)
        assert.deepEqual(measured, rows[id], `${id}: executions disagree`);
      rows[id] = measured;
    }
  }
  return rows;
}

const signed = (value: number) =>
  value > 0
    ? `+${value.toLocaleString("en-US")}`
    : value < 0
      ? `−${(-value).toLocaleString("en-US")}`
      : "0";
const percent = (delta: number, base: number) =>
  base === 0 || delta === 0
    ? ""
    : ` (${delta > 0 ? "+" : delta < 0 ? "−" : ""}${Math.abs((100 * delta) / base).toFixed(3)}%)`;
const count = (value: number | undefined) =>
  value === undefined ? "—" : value.toLocaleString("en-US");

/** Rows whose counts differ between two sides. */
export function movedRows(a: Side, b: Side) {
  const ids = [...new Set([...Object.keys(a.rows), ...Object.keys(b.rows)])].sort();
  return ids.filter((id) => {
    const left = a.rows[id];
    const right = b.rows[id];
    if (left === undefined || right === undefined) return true;
    assert.deepEqual(
      {
        fixture_sha256: right.fixture_sha256,
        window: right.window,
        methodology: right.methodology,
      },
      {
        fixture_sha256: left.fixture_sha256,
        window: left.window,
        methodology: left.methodology,
      },
      `${id}: instruction workload identity changed`,
    );
    return left.instructions !== right.instructions;
  });
}

/** Reject a measurement whose independent rebuild changes any measured value. */
export function assertTwin(label: string, primary: Side, twin: Side) {
  assert.deepEqual(
    movedRows(primary, twin),
    [],
    `${label} instruction counts are not reproducible`,
  );
  assert.deepEqual(twin.sizes, primary.sizes, `${label} binary sizes are not reproducible`);
  assert.deepEqual(
    twin.binaries,
    primary.binaries,
    `${label} benchmark executables are not byte-identical`,
  );
}

export function validateSha(label: string, value: string | undefined) {
  assert.match(value ?? "", /^[0-9a-f]{40}$/, `${label} must be an exact commit SHA`);
}

/** The ready-to-paste markdown report. Twins are optional. */
export function renderReport(input: {
  baseSha: string;
  headSha: string;
  sourceDateEpoch?: string;
  base: Side;
  head: Side;
  baseTwin?: Side;
  headTwin?: Side;
}) {
  const { base, head, baseTwin, headTwin } = input;
  const moved = movedRows(base, head);
  const total = new Set([...Object.keys(base.rows), ...Object.keys(head.rows)]).size;
  const up = moved.filter((id) => {
    const before = base.rows[id]?.instructions;
    const after = head.rows[id]?.instructions;
    return after !== undefined && (before === undefined || after > before);
  });
  const lines: string[] = [];
  lines.push("## Release-profile measurement (fat LTO, 1 codegen unit)", "");
  lines.push(`Base \`${input.baseSha.slice(0, 10)}\` · Head \`${input.headSha.slice(0, 10)}\``, "");
  if (input.sourceDateEpoch !== undefined)
    lines.push(
      `Every build uses \`SOURCE_DATE_EPOCH=${input.sourceDateEpoch}\` (the base commit time).`,
      "",
    );
  lines.push("### Instruction counts", "");
  const twinCols = baseTwin && headTwin;
  lines.push(
    twinCols ? "| Row | Base | Head | Δ | Base twin | Head twin |" : "| Row | Base | Head | Δ |",
    twinCols ? "| --- | ---: | ---: | ---: | ---: | ---: |" : "| --- | ---: | ---: | ---: |",
  );
  for (const id of moved) {
    const b = base.rows[id]?.instructions;
    const h = head.rows[id]?.instructions;
    const delta =
      b === undefined || h === undefined ? "added/removed" : `${signed(h - b)}${percent(h - b, b)}`;
    const cells = [`\`${id}\``, count(b), count(h), delta];
    if (twinCols)
      cells.push(count(baseTwin.rows[id]?.instructions), count(headTwin.rows[id]?.instructions));
    lines.push(`| ${cells.join(" | ")} |`);
  }
  lines.push(
    `| other ${total - moved.length} rows | | | 0 (identical) |${twinCols ? " | |" : ""}`,
    "",
  );
  lines.push(
    `**${moved.length} moved, ${up.length} up, ${total - moved.length} identical** of ${total} rows.`,
    "",
  );
  if (twinCols) {
    const aaBase = movedRows(base, baseTwin).length;
    const aaHead = movedRows(head, headTwin).length;
    const twinPair = movedRows(baseTwin, headTwin);
    const primaryMoved = new Set(moved);
    const agrees =
      twinPair.length === moved.length &&
      twinPair.every((id) => {
        if (!primaryMoved.has(id)) return false;
        const primaryBefore = base.rows[id]?.instructions;
        const primaryAfter = head.rows[id]?.instructions;
        const twinBefore = baseTwin.rows[id]?.instructions;
        const twinAfter = headTwin.rows[id]?.instructions;
        if (primaryBefore === undefined || twinBefore === undefined)
          return primaryBefore === twinBefore && primaryAfter === twinAfter;
        if (primaryAfter === undefined || twinAfter === undefined)
          return primaryAfter === twinAfter && primaryBefore === twinBefore;
        return twinAfter - twinBefore === primaryAfter - primaryBefore;
      });
    const identical = (primary: Side, twin: Side) =>
      primary.binaries !== undefined &&
      JSON.stringify(primary.binaries) === JSON.stringify(twin.binaries);
    const executables = (primary: Side, twin: Side) =>
      primary.binaries === undefined
        ? ""
        : identical(primary, twin)
          ? " (byte-identical bench executables)"
          : " (bench executables differ)";
    lines.push(
      `A/A: base vs base twin ${aaBase} moved${executables(base, baseTwin)}; ` +
        `head vs head twin ${aaHead} moved${executables(head, headTwin)}; ` +
        `twin pair ${agrees ? "repeats every Δ above" : `moves ${twinPair.length} rows (differs from the primary pair)`}.`,
      "",
    );
  }
  lines.push("### Shipped binary sizes (release profile)", "");
  lines.push(
    twinCols
      ? "| Binary | Base | Head | Δ | Base twin | Head twin |"
      : "| Binary | Base | Head | Δ |",
    twinCols ? "| --- | ---: | ---: | ---: | ---: | ---: |" : "| --- | ---: | ---: | ---: |",
  );
  for (const name of Object.keys(base.sizes)) {
    const b = base.sizes[name]!;
    const h = head.sizes[name]!;
    const cells = [name, count(b), count(h), `${signed(h - b)}${percent(h - b, b)}`];
    if (twinCols) cells.push(count(baseTwin.sizes[name]), count(headTwin.sizes[name]));
    lines.push(`| ${cells.join(" | ")} |`);
  }
  lines.push("");
  return lines.join("\n");
}

function run(executable: string, args: string[], cwd: string, env: NodeJS.ProcessEnv) {
  process.stderr.write(`release-measure: (${cwd}) ${executable} ${args.join(" ")}\n`);
  const result = spawnSync(executable, args, { cwd, env, stdio: "inherit" });
  assert.equal(result.status, 0, `${executable} ${args.join(" ")} failed in ${cwd}`);
}

/** Keep the actual Valgrind executable path independent of report labels. */
export function collectInstructionSuite(
  source: string,
  scratch: string,
  destination: string,
  flags: readonly string[],
  env: NodeJS.ProcessEnv,
  collect: typeof run = run,
): string {
  // The collector puts executable links beside its output directory. Using
  // base/head/base-twin/head-twin there changes argv0 length and guest stack
  // alignment, even when the independently rebuilt executables are identical.
  const capture = join(scratch, "instruction-capture");
  rmSync(capture, { recursive: true, force: true });
  mkdirSync(capture, { recursive: true });
  const dir = join(capture, "measurement");
  const failures: unknown[] = [];
  try {
    collect(
      "node",
      ["tools/benchmarks/scripts/instruction-counts.mjs", ...flags, "--collect", "--out", dir],
      source,
      env,
    );
  } catch (error) {
    failures.push(error);
  } finally {
    try {
      if (existsSync(dir))
        cpSync(dir, destination, { recursive: true, force: false, errorOnExist: true });
      const probes = join(
        capture,
        flags.includes("--formatter") ? "formatter-instruction-probes" : "instruction-probes",
      );
      retainMeasuredProbes(dir, probes, join(destination, "probes"));
    } catch (error) {
      failures.push(error);
    } finally {
      try {
        rmSync(capture, { recursive: true, force: true });
      } catch (error) {
        failures.push(error);
      }
    }
  }
  if (failures.length === 1) throw failures[0];
  if (failures.length > 1)
    throw new AggregateError(failures, "instruction collection or evidence retention failed");
  return destination;
}

/**
 * Build and measure one side into `<out>/<label>`. Every side builds in the
 * same, freshly emptied target directory with the pair's pinned build clock.
 */
export function measureSide(
  source: string,
  out: string,
  label: string,
  scratch: string,
  sourceDateEpoch: string,
): Side {
  const target = join(scratch, "target");
  rmSync(target, { recursive: true, force: true });
  const side = join(out, label);
  mkdirSync(side, { recursive: true });
  const env = {
    ...process.env,
    CARGO_TARGET_DIR: target,
    CARGO_INCREMENTAL: "0",
    SOURCE_DATE_EPOCH: sourceDateEpoch,
    // The benches build through [profile.ci-opt]; give it the release codegen.
    CARGO_PROFILE_CI_OPT_LTO: "fat",
    CARGO_PROFILE_CI_OPT_CODEGEN_UNITS: "1",
  };
  const rows: Record<string, InstructionRow> = {};
  const binaries: Record<string, string> = {};
  for (const [suite, flags] of SUITES) {
    const dir = collectInstructionSuite(
      source,
      scratch,
      join(side, suite, "measurement"),
      flags,
      env,
    );
    const folded = foldRuns(JSON.parse(readFileSync(join(dir, "measurement.json"), "utf8")));
    for (const [id, value] of Object.entries(folded)) {
      assert.ok(rows[id] === undefined, `duplicate row ${id}`);
      rows[id] = value;
    }
    const built = JSON.parse(readFileSync(join(dir, "binaries.json"), "utf8")) as Array<{
      pkg: string;
      bench: string;
      sha256: string;
    }>;
    for (const { pkg, bench, sha256 } of built) {
      const id = `${pkg}/${bench}`;
      assert.ok(binaries[id] === undefined, `duplicate benchmark executable ${id}`);
      binaries[id] = sha256;
    }
  }
  const release = { ...env, VIZE_NATIVE_BUILD_TOKEN: "0".repeat(32) };
  const linuxTarget = "x86_64-unknown-linux-gnu";
  const linuxFloorTarget = glibcFloorTarget(linuxTarget);
  run(
    "cargo",
    ["zigbuild", "--locked", "--release", "-p", "vize", "--target", linuxFloorTarget],
    source,
    release,
  );
  run(
    "cargo",
    [
      "zigbuild",
      "--locked",
      "--release",
      "-p",
      "vize_vitrine",
      "--lib",
      "--features",
      "napi,legacy",
      "--target",
      linuxFloorTarget,
    ],
    source,
    release,
  );
  const sizes = {
    "`vize` CLI (stripped)": statSync(join(target, linuxTarget, "release", "vize")).size,
    "`vize_vitrine` N-API library": statSync(
      join(target, linuxTarget, "release", "libvize_vitrine.so"),
    ).size,
  };
  retainSourceCli(join(target, linuxTarget, "release", "vize"), side, gitHead(source));
  const result: Side = { rows, sizes, binaries };
  writeFileSync(join(side, "side.json"), JSON.stringify(result, null, 2) + "\n");
  rmSync(target, { recursive: true, force: true });
  return result;
}

function option(args: string[], name: string) {
  const index = args.indexOf(name);
  return index < 0 ? undefined : args[index + 1];
}

function git(dir: string, ...args: string[]) {
  const result = spawnSync("git", ["-C", dir, ...args], { encoding: "utf8" });
  assert.equal(result.status, 0, `git ${args.join(" ")} failed in ${dir}`);
  return result.stdout.trim();
}

const gitHead = (dir: string) => git(dir, "rev-parse", "HEAD");
const commitTime = (dir: string) => git(dir, "show", "-s", "--format=%ct", "HEAD");

function load(out: string, label: string): Side | undefined {
  const path = join(out, label, "side.json");
  return existsSync(path) ? (JSON.parse(readFileSync(path, "utf8")) as Side) : undefined;
}

function report(out: string) {
  const meta = JSON.parse(readFileSync(join(out, "pair.json"), "utf8"));
  const base = load(out, "base");
  const head = load(out, "head");
  assert.ok(base && head, "base and head measurements are required");
  const markdown = renderReport({
    ...meta,
    base,
    head,
    baseTwin: load(out, "base-twin"),
    headTwin: load(out, "head-twin"),
  });
  writeFileSync(join(out, "summary.md"), markdown);
  process.stdout.write(markdown);
  const baseTwin = load(out, "base-twin");
  const headTwin = load(out, "head-twin");
  if (baseTwin && headTwin) {
    assertTwin("base", base, baseTwin);
    assertTwin("head", head, headTwin);
  }
}

export function measurePair(
  base: string,
  head: string,
  out: string,
  scratch: string,
  twins: boolean,
  measure: typeof measureSide = measureSide,
  resolveHead: typeof gitHead = gitHead,
  resolveEpoch: typeof commitTime = commitTime,
) {
  const pair = join(out, "pair.json");
  assert.ok(!existsSync(pair), `refusing to reuse measurement output ${pair}`);
  mkdirSync(out, { recursive: true });
  const sourceDateEpoch = resolveEpoch(base);
  assert.match(sourceDateEpoch, /^\d+$/, "base commit time must be a Unix timestamp");
  writeFileSync(
    pair,
    JSON.stringify(
      { baseSha: resolveHead(base), headSha: resolveHead(head), sourceDateEpoch },
      null,
      2,
    ) + "\n",
  );
  measure(base, out, "base", scratch, sourceDateEpoch);
  measure(head, out, "head", scratch, sourceDateEpoch);
  if (twins) {
    measure(base, out, "base-twin", scratch, sourceDateEpoch);
    measure(head, out, "head-twin", scratch, sourceDateEpoch);
  }
  report(out);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const reportOnly = option(args, "--report");
  const publish = option(args, "--publish");
  if (args.includes("--validate-inputs")) {
    validateSha("BASE_SHA", process.env.BASE_SHA);
    validateSha("HEAD_SHA", process.env.HEAD_SHA);
  } else if (publish) {
    const summary = join(resolve(publish), "summary.md");
    const destination = process.env.GITHUB_STEP_SUMMARY;
    if (existsSync(summary) && destination) appendFileSync(destination, readFileSync(summary));
    publishMovedAttribution(resolve(publish), movedRows);
  } else if (reportOnly) {
    report(resolve(reportOnly));
  } else {
    const actions = args.includes("--actions");
    const runnerTemp = process.env.RUNNER_TEMP;
    if (actions) assert.ok(runnerTemp, "RUNNER_TEMP is required in Actions");
    const base = resolve(
      actions ? "base" : (option(args, "--base") ?? assert.fail("--base <dir> is required")),
    );
    const head = resolve(
      actions ? "head" : (option(args, "--head") ?? assert.fail("--head <dir> is required")),
    );
    const out = resolve(
      actions
        ? join(runnerTemp!, "release-measure/out")
        : (option(args, "--out") ?? assert.fail("--out <dir> is required")),
    );
    const scratch = resolve(option(args, "--scratch") ?? join(out, ".."));
    const twins = actions ? process.env.TWINS !== "false" : !args.includes("--no-twins");
    measurePair(base, head, out, scratch, twins);
  }
}
