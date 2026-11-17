/**
 * Measures rustc's parallel frontend (`-Zunstable-options --jobs-frontend=N`)
 * against the pinned stable toolchain on the vize workspace.
 *
 * Usage:
 *   node tools/support/github/rust-frontend-threads-bench.ts run \
 *     --scenario <id> --configs stable,j1,j2,j8 --rounds 3 --out <file.json>
 *   node tools/support/github/rust-frontend-threads-bench.ts summarize <file.json>...
 *
 * Scenarios:
 *   ws-check | ws-clippy | ws-build | ws-test  whole workspace from an empty target directory
 *   crate:<package>[:opt|:check]               one crate recompiled with its dependencies warm
 *   incremental                                dev-profile edit/rebuild loop of vize_canon -> vize
 *   determinism                                release binary and diagnostics-order comparison
 *
 * `stable` uses BENCH_STABLE_TOOLCHAIN (default 1.99.1); `jN` uses
 * BENCH_NIGHTLY_TOOLCHAIN (default nightly-2026-11-16) with N frontend threads.
 * Each round visits every configuration in a rotated order on the same machine.
 */
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { commandArguments, isEntrypoint, runEntrypoint } from "../command-runtime.ts";
import { type Report, type Sample, summarize } from "./rust-frontend-threads-summary.ts";

interface Config {
  readonly id: string;
  readonly toolchain: string;
  readonly jobsFrontend?: number;
}

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), "../../..");
const stable = process.env.BENCH_STABLE_TOOLCHAIN ?? "1.99.1";
const nightly = process.env.BENCH_NIGHTLY_TOOLCHAIN ?? "nightly-2026-11-16";
const targetBase = process.env.BENCH_TARGET_DIR ?? path.join(os.tmpdir(), "vize-frontend-bench");
const linux = process.platform === "linux";

export function parseConfig(spec: string): Config {
  if (spec === "stable") return { id: "stable", toolchain: stable };
  const match = /^j(\d+)$/u.exec(spec);
  if (!match) throw new Error(`unknown configuration ${spec}; use stable or jN`);
  const jobsFrontend = Number(match[1]);
  return { id: `nightly-j${jobsFrontend}`, toolchain: nightly, jobsFrontend };
}

function frontendFlags(config: Config): string[] {
  return config.jobsFrontend === undefined
    ? []
    : ["-Zunstable-options", `--jobs-frontend=${config.jobsFrontend}`];
}

let hostTriple: string | undefined;
function host(toolchain: string): string {
  hostTriple ??= /^host: (.+)$/mu.exec(
    spawnSync("rustc", [`+${toolchain}`, "-vV"], { encoding: "utf8" }).stdout,
  )?.[1];
  if (!hostTriple) throw new Error("cannot read the rustc host triple");
  return hostTriple;
}

/**
 * Adds frontend flags to every crate without replacing existing rustflags:
 * RUSTFLAGS wins over config rustflags, so append to it when the caller set it
 * (the Darwin dev shell does); otherwise merge into the target rustflags array
 * that CI's Wild linker setup writes. The `--config` arguments must follow the
 * subcommand: `cargo --config ... clippy` does not forward them to cargo-clippy.
 */
function graphFlags(config: Config): { args: string[]; env: Record<string, string> } {
  const flags = frontendFlags(config);
  if (flags.length === 0) return { args: [], env: {} };
  const inherited = process.env.RUSTFLAGS;
  if (inherited !== undefined)
    return { args: [], env: { RUSTFLAGS: `${inherited} ${flags.join(" ")}` } };
  const array = `[${flags.map((flag) => `'${flag}'`).join(",")}]`;
  return { args: ["--config", `target.${host(config.toolchain)}.rustflags=${array}`], env: {} };
}

interface Measured {
  readonly wallSeconds: number;
  readonly cpuSeconds: number;
  readonly maxRssMiB: number;
  readonly status: number;
  readonly stderr: string;
}

export function parseTimeOutput(
  stderr: string,
  gnu: boolean,
): Omit<Measured, "wallSeconds" | "status" | "stderr"> {
  if (gnu) {
    const user = Number(/User time \(seconds\): ([\d.]+)/u.exec(stderr)?.[1] ?? Number.NaN);
    const system = Number(/System time \(seconds\): ([\d.]+)/u.exec(stderr)?.[1] ?? Number.NaN);
    const rss = Number(
      /Maximum resident set size \(kbytes\): (\d+)/u.exec(stderr)?.[1] ?? Number.NaN,
    );
    return { cpuSeconds: user + system, maxRssMiB: rss / 1024 };
  }
  const times = /([\d.]+) real\s+([\d.]+) user\s+([\d.]+) sys/u.exec(stderr);
  const rss = Number(/(\d+)\s+maximum resident set size/u.exec(stderr)?.[1] ?? Number.NaN);
  return { cpuSeconds: Number(times?.[2]) + Number(times?.[3]), maxRssMiB: rss / 1024 / 1024 };
}

function cargo(
  toolchain: string,
  args: readonly string[],
  env: Record<string, string> = {},
): Measured {
  const started = process.hrtime.bigint();
  const child = spawnSync(
    "/usr/bin/time",
    [linux ? "-v" : "-l", "cargo", `+${toolchain}`, ...args],
    {
      cwd: root,
      env: { ...process.env, CARGO_TERM_COLOR: "never", ...env },
      encoding: "utf8",
      maxBuffer: 1 << 30,
      stdio: ["ignore", "ignore", "pipe"],
    },
  );
  const wallSeconds = Number(process.hrtime.bigint() - started) / 1e9;
  const stderr = child.stderr ?? "";
  const status = child.status ?? 1;
  if (status !== 0) process.stderr.write(stderr.slice(-8000));
  return { wallSeconds, status, stderr, ...parseTimeOutput(stderr, linux) };
}

function prepare(
  toolchain: string,
  args: readonly string[],
  env: Record<string, string> = {},
): void {
  const result = cargo(toolchain, args, env);
  if (result.status !== 0) throw new Error(`cargo +${toolchain} ${args.join(" ")} failed`);
}

function rotate<T>(values: readonly T[], round: number): T[] {
  const offset = round % values.length;
  return [...values.slice(offset), ...values.slice(0, offset)];
}

function sha256(file: string): string | undefined {
  if (!fs.existsSync(file)) return undefined;
  return createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function touch(file: string): void {
  const now = new Date();
  fs.utimesSync(path.join(root, file), now, now);
}

const workspaceCommands: Record<string, string[]> = {
  "ws-check": ["check", "--workspace", "--locked"],
  "ws-clippy": ["clippy", "--workspace", "--locked", "--profile", "ci"],
  "ws-build": ["build", "--workspace", "--locked", "--profile", "ci"],
  "ws-test": ["test", "--workspace", "--no-run", "--locked", "--profile", "ci"],
};

function record(
  samples: Sample[],
  base: Omit<Sample, keyof Measured | "sha256"> & { sha256?: string },
  measured: Measured,
): void {
  const { stderr: _stderr, ...numbers } = measured;
  const sample = { ...base, ...numbers };
  samples.push(sample);
  console.log(JSON.stringify(sample));
}

function runWorkspace(
  scenario: string,
  configs: readonly Config[],
  rounds: number,
  samples: Sample[],
): void {
  const args = workspaceCommands[scenario]!;
  for (const toolchain of new Set(configs.map((config) => config.toolchain))) {
    prepare(toolchain, ["fetch", "--locked"]);
  }
  const target = `${targetBase}-ws`;
  for (let round = 0; round < rounds; round += 1) {
    for (const config of rotate(configs, round)) {
      fs.rmSync(target, { recursive: true, force: true });
      const flags = graphFlags(config);
      const measured = cargo(config.toolchain, [args[0]!, ...flags.args, ...args.slice(1)], {
        ...flags.env,
        CARGO_TARGET_DIR: target,
      });
      const binary = scenario === "ws-build" ? sha256(path.join(target, "ci", "vize")) : undefined;
      record(
        samples,
        { scenario, measure: scenario, config: config.id, round, sha256: binary },
        measured,
      );
    }
  }
  fs.rmSync(target, { recursive: true, force: true });
}

const crateSources: Record<string, string> = {
  vize: "crates/vize/src/lib.rs",
  vize_canon: "crates/vize_canon/src/lib.rs",
  vize_maestro: "crates/vize_maestro/src/lib.rs",
  vize_patina: "crates/vize_patina/src/lib.rs",
};

/**
 * `cargo rustc` passes the frontend flags to the selected crate only, so every
 * configuration of one toolchain shares the same warm dependency graph and the
 * measurement is exactly one rustc invocation.
 */
function runCrate(
  scenario: string,
  configs: readonly Config[],
  rounds: number,
  samples: Sample[],
): void {
  const [, crate, variant] = scenario.split(":");
  const source = crateSources[crate!];
  if (!source) throw new Error(`unknown crate ${crate}`);
  const check: [string, string[]] = ["check", ["--profile", "check"]];
  const measures: Array<[string, string[]]> =
    variant === "opt"
      ? [["opt", ["--profile", "ci-opt"]]]
      : variant === "check"
        ? [check]
        : [check, ["build", ["--profile", "ci"]]];
  const env = { CARGO_INCREMENTAL: "0" };
  const targets = new Map<string, string>();
  for (const toolchain of new Set(configs.map((config) => config.toolchain))) {
    const target = `${targetBase}-${toolchain}`;
    targets.set(toolchain, target);
    for (const [, profile] of measures) {
      prepare(toolchain, ["rustc", "--locked", "-p", crate!, "--lib", ...profile], {
        ...env,
        CARGO_TARGET_DIR: target,
      });
    }
  }
  for (let round = 0; round < rounds; round += 1) {
    for (const config of rotate(configs, round)) {
      for (const [measure, profile] of measures) {
        touch(source);
        const args = [
          "rustc",
          "--locked",
          "-p",
          crate!,
          "--lib",
          ...profile,
          "--",
          ...frontendFlags(config),
        ];
        const measured = cargo(config.toolchain, args, {
          ...env,
          CARGO_TARGET_DIR: targets.get(config.toolchain)!,
        });
        record(
          samples,
          { scenario, measure: `${crate} ${measure}`, config: config.id, round },
          measured,
        );
      }
    }
  }
}

/** Adds one public item per iteration so dependents of vize_canon really rebuild. */
function runIncremental(configs: readonly Config[], rounds: number, samples: Sample[]): void {
  const lib = path.join(root, "crates/vize_canon/src/lib.rs");
  const original = fs.readFileSync(lib, "utf8");
  const graph = ["-p", "vize", "--bin", "vize", "--locked"];
  let edit = 0;
  try {
    for (const config of configs) {
      const flags = graphFlags(config);
      const env = { ...flags.env, CARGO_TARGET_DIR: `${targetBase}-inc-${config.id}` };
      prepare(config.toolchain, ["build", ...flags.args, ...graph], env);
      prepare(config.toolchain, ["check", ...flags.args, ...graph], env);
    }
    for (let round = 0; round < rounds; round += 1) {
      for (const config of rotate(configs, round)) {
        edit += 1;
        fs.writeFileSync(
          lib,
          `${original}\n#[doc(hidden)]\npub fn frontend_bench_probe() -> u32 {\n    ${edit}\n}\n`,
        );
        const flags = graphFlags(config);
        const env = { ...flags.env, CARGO_TARGET_DIR: `${targetBase}-inc-${config.id}` };
        for (const command of ["check", "build"]) {
          const measured = cargo(config.toolchain, [command, ...flags.args, ...graph], env);
          record(
            samples,
            {
              scenario: "incremental",
              measure: `edit vize_canon -> ${command} vize`,
              config: config.id,
              round,
            },
            measured,
          );
        }
      }
    }
  } finally {
    fs.writeFileSync(lib, original);
  }
}

function hashArtifacts(directory: string): Record<string, string> {
  const hashes: Record<string, string> = {};
  const visit = (current: string) => {
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const file = path.join(current, entry.name);
      if (entry.isDirectory()) visit(file);
      else if (/\.(?:rlib|rmeta)$/u.test(entry.name) || file === path.join(directory, "vize")) {
        hashes[path.relative(directory, file)] = sha256(file)!;
      }
    }
  };
  if (fs.existsSync(directory)) visit(directory);
  return hashes;
}

function differing(left: Record<string, string>, right: Record<string, string>): string[] {
  const keys = new Set([...Object.keys(left), ...Object.keys(right)]);
  return [...keys].filter((key) => left[key] !== right[key]).sort();
}

function diagnostics(stderr: string): string[] {
  return stderr.split("\n").filter((line) => line.startsWith("crates/vize_canon/"));
}

function firstDivergence(left: readonly string[], right: readonly string[]): number {
  const length = Math.max(left.length, right.length);
  for (let index = 0; index < length; index += 1) if (left[index] !== right[index]) return index;
  return -1;
}

/**
 * Builds the shipped release binary repeatedly from an empty target directory
 * at the same path, then compares the vize_canon pedantic-Clippy stream order.
 */
function runDeterminism(builds: readonly Config[], samples: Sample[]): Record<string, unknown> {
  const target = `${targetBase}-release`;
  const artifacts: Array<{ config: string; hashes: Record<string, string> }> = [];
  builds.forEach((config, round) => {
    fs.rmSync(target, { recursive: true, force: true });
    const flags = graphFlags(config);
    const args = ["build", ...flags.args, "--release", "--locked", "-p", "vize", "--bin", "vize"];
    const measured = cargo(config.toolchain, args, { ...flags.env, CARGO_TARGET_DIR: target });
    const hashes = hashArtifacts(path.join(target, "release"));
    artifacts.push({ config: config.id, hashes });
    record(
      samples,
      {
        scenario: "determinism",
        measure: "vize release build",
        config: config.id,
        round,
        sha256: hashes.vize,
      },
      measured,
    );
  });
  fs.rmSync(target, { recursive: true, force: true });
  const comparisons = artifacts.slice(1).map((entry, index) => ({
    left: `${artifacts[0]!.config}#0`,
    right: `${entry.config}#${index + 1}`,
    binaryEqual: artifacts[0]!.hashes.vize === entry.hashes.vize,
    artifacts: Object.keys(entry.hashes).length,
    differing: differing(artifacts[0]!.hashes, entry.hashes),
  }));

  const lintTarget = `${targetBase}-diagnostics`;
  const lint = [
    "clippy",
    "--locked",
    "-p",
    "vize_canon",
    "--lib",
    "--profile",
    "ci",
    "--message-format=short",
    "--",
    "-W",
    "clippy::pedantic",
  ];
  const streams: Array<{ config: string; lines: string[] }> = [];
  const parallel = builds.find((build) => (build.jobsFrontend ?? 0) > 1) ?? builds[0]!;
  const serial = builds.filter((build) => build.jobsFrontend === 1).slice(0, 1);
  for (const config of [parallel, parallel, parallel, ...serial]) {
    const flags = graphFlags(config);
    const env = { ...flags.env, CARGO_TARGET_DIR: `${lintTarget}-${config.id}` };
    touch(crateSources.vize_canon!);
    const measured = cargo(config.toolchain, [lint[0]!, ...flags.args, ...lint.slice(1)], env);
    streams.push({ config: config.id, lines: diagnostics(measured.stderr) });
  }
  const reference = streams[0]!;
  const sorted = (lines: readonly string[]) => [...lines].sort();
  return {
    release: comparisons,
    diagnostics: streams.slice(1).map((stream, index) => ({
      left: `${reference.config}#0`,
      right: `${stream.config}#${index + 1}`,
      lines: [reference.lines.length, stream.lines.length],
      sameOrder: firstDivergence(reference.lines, stream.lines) === -1,
      firstDivergence: firstDivergence(reference.lines, stream.lines),
      sameMultiset: firstDivergence(sorted(reference.lines), sorted(stream.lines)) === -1,
    })),
  };
}

/** Whether `run --scenario` accepts this scenario id. */
export function isScenario(scenario: string): boolean {
  const [kind, crate, variant] = scenario.split(":");
  if (kind === "crate")
    return crate! in crateSources && [undefined, "opt", "check"].includes(variant);
  return scenario in workspaceCommands || scenario === "incremental" || scenario === "determinism";
}

function option(args: readonly string[], name: string, fallback?: string): string {
  const index = args.indexOf(name);
  const value = index >= 0 ? args[index + 1] : fallback;
  if (value === undefined) throw new Error(`missing ${name}`);
  return value;
}

function run(args: readonly string[]): number {
  const scenario = option(args, "--scenario");
  const configs = option(args, "--configs").split(",").map(parseConfig);
  const rounds = Number(option(args, "--rounds", "3"));
  const out = option(args, "--out");
  const samples: Sample[] = [];
  let determinism: Record<string, unknown> | undefined;
  if (scenario in workspaceCommands) runWorkspace(scenario, configs, rounds, samples);
  else if (scenario.startsWith("crate:")) runCrate(scenario, configs, rounds, samples);
  else if (scenario === "incremental") runIncremental(configs, rounds, samples);
  else if (scenario === "determinism") determinism = runDeterminism(configs, samples);
  else throw new Error(`unknown scenario ${scenario}`);
  const report: Report = {
    host: {
      cpus: os.availableParallelism(),
      memoryGiB: os.totalmem() / 2 ** 30,
      platform: `${process.platform}-${process.arch}`,
      loadAverage: os.loadavg(),
    },
    samples,
    determinism,
  };
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.writeFileSync(out, `${JSON.stringify(report, null, 2)}\n`);
  return samples.every((sample) => sample.status === 0) ? 0 : 1;
}

export function main(args: readonly string[]): number {
  const [command, ...rest] = args;
  if (command === "run") return run(rest);
  if (command === "summarize") {
    const reports = rest.map((file) => JSON.parse(fs.readFileSync(file, "utf8")) as Report);
    console.log(summarize(reports));
    return 0;
  }
  throw new Error("usage: rust-frontend-threads-bench.ts run|summarize ...");
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
