/**
 * Published-package mapper qualification inside WSL2. Each corpus is prepared
 * independently on the distro filesystem and a Windows-mounted filesystem;
 * installs are untimed, while cold/warm process wall time and peak RSS are
 * measured in alternating lane order. Target misses are reported; correctness
 * refusals fail the command.
 */

import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { availableParallelism } from "node:os";
import { join, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { parseArgs } from "node:util";

import {
  type ScaleProject,
  generateScaleProject,
  monorepoMapperConfigs,
  monorepoOutDir,
  writeProject,
} from "./scale-project-generator.ts";
import { gate, parseRun, refusalText } from "./typecheck-scaling-gate.ts";

const HOSTS = {
  typescript: "7.1.0-beta",
  tsrs: "0.20.1-ts7.1.0-beta",
  vueMapper: "4.0.0-alpha.3",
};
const TARGETS = { mapperSpeedup: 100, memoryReduction: 10, tsrsMs: 500 };
const mapperTsconfigName = (mapper: "vize" | "vue") => `tsconfig.${mapper}-mapper.json`;

type LaneId =
  | "tsc-vize-mapper"
  | "tsc-vue-mapper"
  | "tsrs-fast-vize-mapper"
  | "tsrs-fast-parallel-vize-mapper";
type Sample = {
  location: string;
  profile: string;
  lane: LaneId;
  phase: "cold" | "warm";
  run: number;
  wallMs: number;
  maxRssKb: number;
  fingerprint: string;
  errors: number;
  refused: string | null;
};

const median = (values: number[]): number | null => {
  if (!values.length) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle]! : (sorted[middle - 1]! + sorted[middle]!) / 2;
};
const divide = (numerator: number | null, denominator: number | null) =>
  numerator != null && denominator != null && denominator > 0 ? numerator / denominator : null;
const fixed = (value: number | null, digits = 2) => (value == null ? "n/a" : value.toFixed(digits));
const runOrder = <T>(values: T[], index: number) =>
  index % 2 === 0 ? [...values] : [...values].reverse();

function install(dir: string, vize: string, vue: string): void {
  writeFileSync(
    join(dir, "package.json"),
    `${JSON.stringify(
      {
        private: true,
        dependencies: {
          "@vue/content-mapper": HOSTS.vueMapper,
          tsrs: HOSTS.tsrs,
          typescript: HOSTS.typescript,
          vize,
          vue,
        },
      },
      null,
      2,
    )}\n`,
  );
  const result = spawnSync("npm", ["install", "--no-audit", "--no-fund"], {
    cwd: dir,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 600_000,
  });
  if (result.status !== 0)
    throw new Error(`npm install failed in ${dir}\n${result.stdout}\n${result.stderr}`);
}

function prepare(dir: string, corpus: ScaleProject, vize: string, vue: string): void {
  writeProject(dir, corpus.files);
  install(dir, vize, vue);
  if (corpus.kind === "monorepo") {
    for (const mapper of [
      { id: "vize", package: "vize" },
      { id: "vue", package: "@vue/content-mapper" },
    ] as const) {
      writeProject(
        dir,
        monorepoMapperConfigs(corpus, mapper, mapperTsconfigName(mapper.id)),
        false,
      );
    }
  } else {
    for (const mapper of [
      { id: "vize", package: "vize" },
      { id: "vue", package: "@vue/content-mapper" },
    ] as const) {
      writeFileSync(
        join(dir, mapperTsconfigName(mapper.id)),
        `${JSON.stringify(
          {
            extends: "./tsconfig.json",
            contentMappers: [{ package: mapper.package, extensions: [".vue"] }],
          },
          null,
          2,
        )}\n`,
      );
    }
  }
}

function clearBuildOutputs(dir: string, corpus: ScaleProject): void {
  for (const name of corpus.packages) {
    for (const entry of readdirSync(join(dir, name))) {
      if (entry.startsWith(monorepoOutDir("")))
        rmSync(join(dir, name, entry), { recursive: true, force: true });
    }
  }
}

function invoke(dir: string, corpus: ScaleProject, lane: LaneId): { bin: string; args: string[] } {
  const host = lane.startsWith("tsrs") ? "tsrs" : "typescript";
  const bin = join(dir, "node_modules", host, "bin", host === "tsrs" ? "tsrs" : "tsc");
  const mapper = lane === "tsc-vue-mapper" ? "vue" : "vize";
  const extra = lane.startsWith("tsrs-fast")
    ? [
        "--fast",
        ...(lane === "tsrs-fast-parallel-vize-mapper"
          ? ["--checkers", String(availableParallelism())]
          : []),
      ]
    : [];
  return {
    bin,
    args:
      corpus.kind === "monorepo"
        ? [
            "-b",
            join(dir, mapperTsconfigName(mapper)),
            "--force",
            ...extra,
            "--runExternalCode",
            "--pretty",
            "false",
          ]
        : [
            "--noEmit",
            ...extra,
            "--runExternalCode",
            "--pretty",
            "false",
            "-p",
            join(dir, mapperTsconfigName(mapper)),
          ],
  };
}

function measure(
  location: string,
  profile: string,
  dir: string,
  corpus: ScaleProject,
  lane: LaneId,
  phase: Sample["phase"],
  run: number,
): Sample {
  clearBuildOutputs(dir, corpus);
  const command = invoke(dir, corpus, lane);
  const timeFile = join(dir, `.time-${lane}`);
  const started = performance.now();
  const result = spawnSync(
    "/usr/bin/time",
    ["-q", "-f", "%e\\t%M", "-o", timeFile, command.bin, ...command.args],
    {
      cwd: dir,
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 1024,
      timeout: 1_800_000,
      env: { ...process.env, NO_COLOR: "1", FORCE_COLOR: "0" },
    },
  );
  const wallMs = performance.now() - started;
  const [, rssText] = readFileSync(timeFile, "utf8").trim().split("\t");
  let fingerprint = "";
  let errors = 0;
  let refused: string | null = null;
  try {
    if (result.error) throw result.error;
    const parsed = parseRun(
      { status: result.status, stdout: result.stdout, stderr: result.stderr },
      dir,
      "text",
    );
    const verdict = gate(lane, parsed.diagnostics, corpus.plants);
    if (!verdict.ok) throw new Error(refusalText(lane, verdict));
    fingerprint = parsed.fingerprint;
    errors = verdict.errors;
  } catch (error) {
    refused = error instanceof Error ? error.message : String(error);
  }
  return {
    location,
    profile,
    lane,
    phase,
    run,
    wallMs,
    maxRssKb: Number(rssText),
    fingerprint,
    errors,
    refused,
  };
}

function summary(samples: Sample[]): string {
  const keys = [...new Set(samples.map((sample) => `${sample.location}\0${sample.profile}`))];
  const lines = [
    "# WSL2 mapper qualification",
    "",
    `Targets: ${TARGETS.mapperSpeedup}x mapper wall speedup, ${TARGETS.memoryReduction}x peak-RSS reduction, and tsrs --fast at or below ${TARGETS.tsrsMs}ms. Cold and warm results and exact diagnostic fingerprints must all pass independently.`,
    "",
    "| location | profile | diagnostics | mapper wall cold / warm | memory cold / warm | tsrs cold / warm | verdict |",
    "| --- | --- | --- | ---: | ---: | ---: | --- |",
  ];
  for (const key of keys) {
    const [location, profile] = key.split("\0") as [string, string];
    const mine = samples.filter(
      (sample) => sample.location === location && sample.profile === profile,
    );
    const lane = (id: LaneId, phase: Sample["phase"]) =>
      mine.filter((sample) => sample.lane === id && sample.phase === phase && !sample.refused);
    const metric = (id: LaneId, phase: Sample["phase"], field: "wallMs" | "maxRssKb") =>
      median(lane(id, phase).map((sample) => sample[field]));
    const fingerprints = (id: LaneId) =>
      new Set(
        mine
          .filter((sample) => sample.lane === id && !sample.refused)
          .map((sample) => sample.fingerprint),
      );
    const vizeFingerprint = [...fingerprints("tsc-vize-mapper")];
    const vueFingerprint = [...fingerprints("tsc-vue-mapper")];
    const tsrsFingerprint = [...fingerprints("tsrs-fast-parallel-vize-mapper")];
    const diagnosticParity =
      vizeFingerprint.length === 1 &&
      vueFingerprint.length === 1 &&
      tsrsFingerprint.length === 1 &&
      vizeFingerprint[0] === vueFingerprint[0] &&
      vizeFingerprint[0] === tsrsFingerprint[0];
    const coldSpeedup = divide(
      metric("tsc-vue-mapper", "cold", "wallMs"),
      metric("tsc-vize-mapper", "cold", "wallMs"),
    );
    const warmSpeedup = divide(
      metric("tsc-vue-mapper", "warm", "wallMs"),
      metric("tsc-vize-mapper", "warm", "wallMs"),
    );
    const coldMemory = divide(
      metric("tsc-vue-mapper", "cold", "maxRssKb"),
      metric("tsc-vize-mapper", "cold", "maxRssKb"),
    );
    const warmMemory = divide(
      metric("tsc-vue-mapper", "warm", "maxRssKb"),
      metric("tsc-vize-mapper", "warm", "maxRssKb"),
    );
    const tsrsCold = metric("tsrs-fast-parallel-vize-mapper", "cold", "wallMs");
    const tsrsWarm = metric("tsrs-fast-parallel-vize-mapper", "warm", "wallMs");
    const refused = mine.filter((sample) => sample.refused).length;
    const met =
      refused === 0 &&
      diagnosticParity &&
      (coldSpeedup ?? 0) >= TARGETS.mapperSpeedup &&
      (warmSpeedup ?? 0) >= TARGETS.mapperSpeedup &&
      (coldMemory ?? 0) >= TARGETS.memoryReduction &&
      (warmMemory ?? 0) >= TARGETS.memoryReduction &&
      (tsrsCold ?? Number.POSITIVE_INFINITY) <= TARGETS.tsrsMs &&
      (tsrsWarm ?? Number.POSITIVE_INFINITY) <= TARGETS.tsrsMs;
    lines.push(
      `| ${location} | ${profile} | ${refused ? `**${refused} refused**` : diagnosticParity ? "exact" : "**mismatch**"} | ×${fixed(coldSpeedup)} / ×${fixed(warmSpeedup)} | ×${fixed(coldMemory)} / ×${fixed(warmMemory)} | ${fixed(tsrsCold, 0)}ms / ${fixed(tsrsWarm, 0)}ms | ${met ? "**met**" : "**miss**"} |`,
    );
  }
  return `${lines.join("\n")}\n`;
}

function main(): void {
  const { values } = parseArgs({
    options: {
      location: { type: "string", multiple: true },
      vize: { type: "string", default: "latest" },
      vue: { type: "string", default: "3.5.42" },
      sizes: { type: "string", default: "1000" },
      "giant-blocks": { type: "string", default: "10000" },
      "monorepo-sfcs": { type: "string", default: "1000" },
      "monorepo-packages": { type: "string", default: "6" },
      runs: { type: "string", default: "3" },
      out: { type: "string" },
    },
  });
  if (!values.location?.length || !values.out)
    throw new Error("at least one --location label=path and --out are required");
  const runs = Number(values.runs);
  if (!Number.isSafeInteger(runs) || runs < 1) throw new Error("--runs must be positive");
  const sizes = values.sizes.split(",").filter(Boolean).map(Number);
  const giantBlocks = Number(values["giant-blocks"]);
  const monorepoSfcs = Number(values["monorepo-sfcs"]);
  const monorepoPackages = Number(values["monorepo-packages"]);
  const corpusFactories: Array<[string, () => ScaleProject]> = [
    ...sizes.map<[string, () => ScaleProject]>((sfcs) => [
      `vue-${sfcs}`,
      () => generateScaleProject({ sfcs, kind: "vue", parityPlants: true }),
    ]),
    ...(giantBlocks > 0
      ? [
          [
            `giant-${giantBlocks}-blocks`,
            () => generateScaleProject({ sfcs: 1, kind: "giant", giantBlocks }),
          ] as [string, () => ScaleProject],
        ]
      : []),
    ...(monorepoSfcs > 0
      ? [
          [
            `monorepo-${monorepoSfcs}`,
            () =>
              generateScaleProject({
                sfcs: monorepoSfcs,
                kind: "monorepo",
                parityPlants: true,
                featurePackages: monorepoPackages,
              }),
          ] as [string, () => ScaleProject],
        ]
      : []),
  ];
  const locations = values.location.map((value) => {
    const at = value.indexOf("=");
    if (at < 1) throw new Error(`invalid --location ${value}`);
    return { label: value.slice(0, at), root: resolve(value.slice(at + 1)) };
  });
  const samples: Sample[] = [];
  const lanes: LaneId[] = [
    "tsc-vize-mapper",
    "tsc-vue-mapper",
    "tsrs-fast-vize-mapper",
    "tsrs-fast-parallel-vize-mapper",
  ];
  for (const [profile, create] of corpusFactories) {
    for (const location of locations) {
      const corpus = create();
      const dir = join(location.root, profile);
      prepare(dir, corpus, values.vize, values.vue);
      for (let index = 0; index <= runs; index += 1) {
        const phase = index === 0 ? "cold" : "warm";
        for (const lane of runOrder(lanes, index + samples.length)) {
          const sample = measure(location.label, profile, dir, corpus, lane, phase, index);
          samples.push(sample);
          console.error(
            `${location.label} ${profile} ${lane} ${phase}: ${sample.wallMs.toFixed(0)}ms ${sample.maxRssKb}K${sample.refused ? ` REFUSED ${sample.refused}` : ""}`,
          );
        }
      }
    }
  }
  const out = resolve(values.out);
  mkdirSync(out, { recursive: true });
  writeFileSync(
    join(out, "results.json"),
    `${JSON.stringify({ schemaVersion: 1, hosts: HOSTS, targets: TARGETS, samples }, null, 2)}\n`,
  );
  writeFileSync(join(out, "summary.md"), summary(samples));
  if (samples.some((sample) => sample.refused)) process.exitCode = 1;
}

main();
