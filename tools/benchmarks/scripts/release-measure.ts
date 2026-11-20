#!/usr/bin/env node
/**
 * Release-profile measurement of a base/head source pair: level instruction
 * counts (stages, formatter, SFC parsing) and the shipped `vize` CLI and
 * `vize_vitrine` N-API library sizes, with a discarded warm-up and an exact
 * independent A/A twin per side.
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
}

export function validateSha(label: string, value: string | undefined) {
  assert.match(value ?? "", /^[0-9a-f]{40}$/, `${label} must be an exact commit SHA`);
}

/** The ready-to-paste markdown report. Twins are optional. */
export function renderReport(input: {
  baseSha: string;
  headSha: string;
  base: Side;
  head: Side;
  baseTwin?: Side;
  headTwin?: Side;
}) {
  const { base, head, baseTwin, headTwin } = input;
  const moved = movedRows(base, head);
  const total = new Set([...Object.keys(base.rows), ...Object.keys(head.rows)]).size;
  const up = moved.filter(
    (id) => (head.rows[id]?.instructions ?? Infinity) > (base.rows[id]?.instructions ?? -Infinity),
  );
  const lines: string[] = [];
  lines.push("## Release-profile measurement (fat LTO, 1 codegen unit)", "");
  lines.push(`Base \`${input.baseSha.slice(0, 10)}\` · Head \`${input.headSha.slice(0, 10)}\``, "");
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
    const agrees =
      twinPair.length === moved.length &&
      twinPair.every(
        (id) =>
          headTwin.rows[id]!.instructions - baseTwin.rows[id]!.instructions ===
          head.rows[id]!.instructions - base.rows[id]!.instructions,
      );
    lines.push(
      `A/A: base vs base twin ${aaBase} moved; head vs head twin ${aaHead} moved; ` +
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

/** Build and measure one side into `<out>/<label>`, in a fresh target directory. */
export function measureSide(source: string, out: string, label: string, scratch: string): Side {
  const target = join(scratch, `target-${label}`);
  rmSync(target, { recursive: true, force: true });
  const side = join(out, label);
  mkdirSync(side, { recursive: true });
  const env = {
    ...process.env,
    CARGO_TARGET_DIR: target,
    CARGO_INCREMENTAL: "0",
    // The benches build through [profile.ci-opt]; give it the release codegen.
    CARGO_PROFILE_CI_OPT_LTO: "fat",
    CARGO_PROFILE_CI_OPT_CODEGEN_UNITS: "1",
  };
  const rows: Record<string, InstructionRow> = {};
  for (const [suite, flags] of SUITES) {
    // instruction-counts.mjs keeps its guest links beside the out directory,
    // so every suite gets its own parent.
    const dir = join(side, suite, "measurement");
    run(
      "node",
      ["tools/benchmarks/scripts/instruction-counts.mjs", ...flags, "--collect", "--out", dir],
      source,
      env,
    );
    const folded = foldRuns(JSON.parse(readFileSync(join(dir, "measurement.json"), "utf8")));
    for (const [id, value] of Object.entries(folded)) {
      assert.ok(rows[id] === undefined, `duplicate row ${id}`);
      rows[id] = value;
    }
  }
  const release = { ...env, VIZE_NATIVE_BUILD_TOKEN: "0".repeat(32) };
  const linuxTarget = "x86_64-unknown-linux-gnu";
  run(
    "cargo",
    ["zigbuild", "--locked", "--release", "-p", "vize", "--target", glibcFloorTarget(linuxTarget)],
    source,
    release,
  );
  run(
    "cargo",
    ["build", "--locked", "--release", "-p", "vize_vitrine", "--lib", "--features", "napi,legacy"],
    source,
    release,
  );
  const sizes = {
    "`vize` CLI (stripped)": statSync(join(target, linuxTarget, "release", "vize")).size,
    "`vize_vitrine` N-API library": statSync(join(target, "release", "libvize_vitrine.so")).size,
  };
  const result: Side = { rows, sizes };
  writeFileSync(join(side, "side.json"), JSON.stringify(result, null, 2) + "\n");
  rmSync(target, { recursive: true, force: true });
  return result;
}

function option(args: string[], name: string) {
  const index = args.indexOf(name);
  return index < 0 ? undefined : args[index + 1];
}

function gitHead(dir: string) {
  const result = spawnSync("git", ["-C", dir, "rev-parse", "HEAD"], { encoding: "utf8" });
  assert.equal(result.status, 0, `git rev-parse failed in ${dir}`);
  return result.stdout.trim();
}

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
) {
  const pair = join(out, "pair.json");
  assert.ok(!existsSync(pair), `refusing to reuse measurement output ${pair}`);
  mkdirSync(out, { recursive: true });
  writeFileSync(
    pair,
    JSON.stringify({ baseSha: resolveHead(base), headSha: resolveHead(head) }, null, 2) + "\n",
  );
  // The first fat-LTO build on a fresh runner can differ from every subsequent
  // rebuild. Prime the toolchain and Cargo caches, then discard all its output.
  measure(base, out, "warmup", scratch);
  rmSync(join(out, "warmup"), { recursive: true, force: true });
  measure(base, out, "base", scratch);
  measure(head, out, "head", scratch);
  if (twins) {
    measure(base, out, "base-twin", scratch);
    measure(head, out, "head-twin", scratch);
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
