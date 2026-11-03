/**
 * Real-corpus type-check pair: misskey and n8n checked by tsc and tsrs
 * (TypeScript 7.1 hosts) through the base and head Vize mappers, with Vue's
 * mapper as the reference.
 *
 *   node typecheck-corpus-pair.ts --base <worktree> --head <worktree> \
 *     --base-sha <sha> --head-sha <sha> --runs 5 --work <dir> --out <dir>
 *
 * Each worktree's `npm/cli` (and the `@vizejs/native` it links) must be built.
 * CPU is user+sys of the whole host process tree, mapper included
 * (typecheck-command.mjs); max RSS is GNU time's %M.
 */

import { spawnSync } from "node:child_process";
import { cpSync, mkdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import {
  GNU_TIME_FORMAT,
  type Diagnostic,
  type GnuTime,
  type StatEntry,
  parseGnuTime,
  parseHostRun,
  runOrder,
  summarizeProbe,
} from "./typecheck-corpus-pair-parse.ts";
import {
  type PairResults,
  renderPairSummary,
  summarizePair,
} from "./typecheck-corpus-pair-report.ts";
import { runTypecheckCommand } from "./typecheck-command.mjs";
import {
  CONTENT_MAPPERS,
  MAPPER_HOSTS,
  type MapperHosts,
  TYPECHECK_HOSTS_DIR,
  mapperTsconfigName,
  prepareMapperPackages,
  resolveMapperHosts,
} from "./typecheck-mapper-hosts.ts";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptsDir, "..", "..", "..");
const CORPUS_PACKAGES_DIR = resolve(scriptsDir, "..", "typecheck-corpora");
const PROBE_SCRIPT = join(scriptsDir, "content-mapper-byte-probe.ts");
const GNU_TIME = "/usr/bin/time";

type Host = (typeof MAPPER_HOSTS)[number];
type Mapper = (typeof CONTENT_MAPPERS)[number];

const shared = {
  strict: true,
  target: "ES2022",
  module: "esnext",
  moduleResolution: "Bundler",
  resolveJsonModule: true,
  isolatedModules: true,
  useDefineForClassFields: true,
  skipLibCheck: true,
  noEmit: true,
  incremental: false,
  allowJs: true,
  paths: { "@/*": ["./src/*"] },
  types: ["node"],
  lib: ["esnext", "dom", "dom.iterable"],
  jsx: "preserve",
};

/** Pinned submodule sources; `sfcs` is what the host must hand the mapper. */
export const CORPORA = [
  {
    id: "misskey",
    label: "misskey frontend",
    source: "tests/_fixtures/_git/misskey/packages/frontend",
    copy: ["src", "@types"],
    sfcs: 583,
    tsconfig: {
      compilerOptions: {
        ...shared,
        noImplicitAny: true,
        noImplicitReturns: true,
        experimentalDecorators: true,
        verbatimModuleSyntax: true,
      },
      include: ["./src/**/*.ts", "./src/**/*.vue", "./@types/**/*.ts"],
      exclude: ["./src/**/*.stories.ts"],
    },
  },
  {
    id: "n8n",
    label: "n8n editor-ui",
    source: "tests/_fixtures/_git/n8n/packages/frontend/editor-ui",
    copy: ["src"],
    sfcs: 1113,
    tsconfig: {
      compilerOptions: shared,
      include: ["./src/**/*.ts", "./src/**/*.vue"],
      exclude: ["./src/**/*.test.ts", "./src/**/*.spec.ts", "./src/**/__tests__/**"],
    },
  },
] as const;
type Corpus = (typeof CORPORA)[number];

/** Vize from the base and the head build, and Vue's mapper beside the base build. */
export const VARIANTS = [
  { id: "vize-base", mapper: "vize", project: "base" },
  { id: "vize-head", mapper: "vize", project: "head" },
  { id: "vue", mapper: "vue", project: "base" },
] as const;
type Variant = (typeof VARIANTS)[number];

const mapperOf = (variant: Variant): Mapper =>
  CONTENT_MAPPERS.find((mapper) => mapper.id === variant.mapper) as Mapper;
const probeName = (mapper: Mapper) => `${mapper.id}-bytes-probe`;
const probeTsconfig = (mapper: Mapper) => `tsconfig.${mapper.id}-probe.json`;

/** A probe package that starts the proxy in place of the real mapper. */
function writeProbePackage(project: string, mapper: Mapper): void {
  const real = realpathSync(join(project, "node_modules", mapper.package));
  const manifest = JSON.parse(readFileSync(join(real, "package.json"), "utf8")) as {
    typescript?: { contentMapper?: { exec?: string[] } };
  };
  const contentMapper = manifest.typescript?.contentMapper;
  if (!contentMapper?.exec?.length) throw new Error(`${mapper.package} declares no content mapper`);
  const dir = join(project, "node_modules", probeName(mapper));
  mkdirSync(dir);
  const exec = ["node", PROBE_SCRIPT, real, "--", ...contentMapper.exec];
  const probe = { name: probeName(mapper), version: "0.0.0", private: true };
  writeFileSync(
    join(dir, "package.json"),
    `${JSON.stringify({ ...probe, typescript: { contentMapper: { ...contentMapper, exec } } })}\n`,
  );
  writeFileSync(
    join(project, probeTsconfig(mapper)),
    `${JSON.stringify({
      extends: "./tsconfig.json",
      contentMappers: [{ package: probeName(mapper), extensions: [".vue"] }],
    })}\n`,
  );
}

/** Corpus sources and pinned packages as real files; mappers as typecheck-mapper-hosts installs them. */
function prepareProject(dir: string, corpus: Corpus, hosts: MapperHosts): void {
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  for (const entry of corpus.copy) {
    cpSync(join(repoRoot, corpus.source, entry), join(dir, entry), {
      recursive: true,
      verbatimSymlinks: true,
    });
  }
  writeFileSync(join(dir, "tsconfig.json"), `${JSON.stringify(corpus.tsconfig, null, 2)}\n`);
  cpSync(join(CORPUS_PACKAGES_DIR, "node_modules"), join(dir, "node_modules"), {
    recursive: true,
    verbatimSymlinks: true,
  });
  prepareMapperPackages(dir, hosts);
  for (const mapper of CONTENT_MAPPERS) writeProbePackage(dir, mapper);
}

type HostRun = {
  wallMs: number;
  cpuMs: number;
  time: GnuTime;
  status: number | null;
  stats: StatEntry[];
  diagnostics: Diagnostic[];
  fingerprint: string;
};

function hostRun(
  host: Host,
  bin: string,
  cwd: string,
  tsconfig: string,
  timeFile: string,
  env: Record<string, string> = {},
): HostRun {
  const args = [
    "--noEmit",
    ...host.args,
    "--runExternalCode",
    "--extendedDiagnostics",
    "--pretty",
    "false",
    "-p",
    join(cwd, tsconfig),
  ];
  const result = runTypecheckCommand(
    GNU_TIME,
    ["-q", "-f", GNU_TIME_FORMAT, "-o", timeFile, bin, ...args],
    { cwd, env },
  ) as { ms: number; cpuMs: number | null; status: number | null; stdout: string; stderr: string };
  if (result.cpuMs == null) throw new Error("process-tree CPU needs Linux /proc/self/stat");
  const time = parseGnuTime(readFileSync(timeFile, "utf8"));
  if (time.status !== result.status) {
    throw new Error(`GNU time saw status ${time.status}, the runner saw ${result.status}`);
  }
  return {
    wallMs: result.ms,
    cpuMs: result.cpuMs,
    time,
    status: result.status,
    ...parseHostRun(result, cwd),
  };
}

function packageVersion(dir: string, name: string): string {
  return (
    JSON.parse(readFileSync(join(dir, "node_modules", name, "package.json"), "utf8")) as {
      version: string;
    }
  ).version;
}

function requireGnuTime(): string {
  const probe = spawnSync(GNU_TIME, ["--version"], { encoding: "utf8" });
  const version = `${probe.stdout ?? ""}${probe.stderr ?? ""}`.trim().split("\n")[0] ?? "";
  if (probe.status !== 0 || !/GNU time/iu.test(version)) {
    throw new Error(`max RSS needs GNU time at ${GNU_TIME}; found: ${version || probe.error}`);
  }
  return version;
}

function main(): void {
  const { values } = parseArgs({
    options: {
      base: { type: "string" },
      head: { type: "string" },
      "base-sha": { type: "string" },
      "head-sha": { type: "string" },
      runs: { type: "string", default: "5" },
      work: { type: "string" },
      out: { type: "string" },
    },
  });
  const runs = Number(values.runs);
  if (!values.base || !values.head || !values.work || !values.out) {
    throw new Error("--base, --head, --work and --out are required");
  }
  if (!Number.isInteger(runs) || runs < 1) throw new Error(`--runs must be a positive integer`);
  const shas = { base: values["base-sha"] ?? "", head: values["head-sha"] ?? "" };
  for (const sha of Object.values(shas)) {
    if (!/^[0-9a-f]{40}$/u.test(sha)) throw new Error(`not a full SHA: ${sha}`);
  }
  const gnuTime = requireGnuTime();
  const builds = {
    base: resolveMapperHosts(TYPECHECK_HOSTS_DIR, join(resolve(values.base), "npm", "cli")),
    head: resolveMapperHosts(TYPECHECK_HOSTS_DIR, join(resolve(values.head), "npm", "cli")),
  };
  const work = resolve(values.work);
  const out = resolve(values.out);
  mkdirSync(join(out, "diagnostics"), { recursive: true });
  mkdirSync(join(work, "time"), { recursive: true });
  const projects = (corpus: Corpus, role: "base" | "head") => join(work, `${corpus.id}-${role}`);
  for (const corpus of CORPORA) {
    for (const role of ["base", "head"] as const)
      prepareProject(projects(corpus, role), corpus, builds[role]);
  }

  const lanes = CORPORA.flatMap((corpus) =>
    MAPPER_HOSTS.flatMap((host) => VARIANTS.map((variant) => ({ corpus, host, variant }))),
  );
  type Lane = (typeof lanes)[number];
  const laneKey = ({ corpus, host, variant }: Lane) => `${corpus.id}-${host.id}-${variant.id}`;
  const bin = (host: Host) => builds.base[host.id];
  const cwd = ({ corpus, variant }: Lane) => projects(corpus, variant.project);

  // Untimed probe first: generated bytes through the proxy, and a warm cache for the timed runs.
  const probes = new Map<string, ReturnType<typeof summarizeProbe> & { fingerprint: string }>();
  for (const lane of lanes) {
    const key = laneKey(lane);
    const record = join(work, "time", `${key}.probe.jsonl`);
    writeFileSync(record, "");
    const run = hostRun(
      lane.host,
      bin(lane.host),
      cwd(lane),
      probeTsconfig(mapperOf(lane.variant)),
      join(work, "time", `${key}.probe.time`),
      {
        VIZE_MAPPER_PROBE_RECORD: record,
      },
    );
    const probe = summarizeProbe(readFileSync(record, "utf8"));
    if (probe.errors.length > 0)
      throw new Error(`${key}: mapper errors\n${probe.errors.join("\n")}`);
    if (probe.files !== lane.corpus.sfcs) {
      throw new Error(
        `${key}: the host mapped ${probe.files} SFCs, the corpus has ${lane.corpus.sfcs}`,
      );
    }
    probes.set(key, { ...probe, fingerprint: run.fingerprint });
    console.log(
      `${key}: probe ${probe.files} SFCs, ${probe.bytes} bytes, ${run.diagnostics.length} diagnostics`,
    );
  }

  const timed = new Map<string, HostRun[]>();
  for (let run = 0; run < runs; run++) {
    for (const corpus of CORPORA) {
      for (const host of MAPPER_HOSTS) {
        for (const variant of runOrder(VARIANTS, run)) {
          const lane = { corpus, host, variant };
          const key = laneKey(lane);
          const result = hostRun(
            host,
            bin(host),
            cwd(lane),
            mapperTsconfigName(mapperOf(variant)),
            join(work, "time", `${key}.${run}.time`),
          );
          timed.set(key, [...(timed.get(key) ?? []), result]);
          console.log(
            `${key} run ${run + 1}/${runs}: cpu ${(result.cpuMs / 1000).toFixed(2)}s rss ${result.time.maxRssKb}K`,
          );
        }
      }
    }
  }

  const results: PairResults = {
    schemaVersion: 1,
    base: { sha: shas.base },
    head: { sha: shas.head },
    runs,
    measurement: {
      runner: process.env.VIZE_PAIR_RUNNER_LABEL ?? "unknown",
      node: process.version,
      gnuTime,
      cpu: "user+sys of every reaped descendant of the runner (cutime+cstime from /proc/self/stat), so the host and every mapper process it waited for",
      maxRss:
        "GNU time %M: ru_maxrss from wait4, the peak RSS of the largest single process in the host's tree",
      packages: {
        typescript: packageVersion(TYPECHECK_HOSTS_DIR, "typescript"),
        tsrs: packageVersion(TYPECHECK_HOSTS_DIR, "tsrs"),
        "@vue/content-mapper": packageVersion(TYPECHECK_HOSTS_DIR, "@vue/content-mapper"),
        vue: packageVersion(CORPUS_PACKAGES_DIR, "vue"),
        "@types/node": packageVersion(CORPUS_PACKAGES_DIR, "@types/node"),
      },
    },
    lanes: lanes.map((lane) => {
      const key = laneKey(lane);
      const runsOf = timed.get(key) ?? [];
      const probe = probes.get(key);
      if (!probe) throw new Error(`${key}: no probe`);
      const diagnostics = (runsOf[0] as HostRun).diagnostics;
      writeFileSync(
        join(out, "diagnostics", `${key}.json`),
        `${JSON.stringify(diagnostics, null, 1)}\n`,
      );
      return {
        corpus: lane.corpus.id,
        corpusLabel: lane.corpus.label,
        sfcs: lane.corpus.sfcs,
        host: lane.host.id,
        variant: lane.variant.id,
        probe,
        diagnostics,
        runs: runsOf.map(({ diagnostics: _diagnostics, ...run }) => run),
      };
    }),
  };
  const summary = summarizePair(results);
  // Full diagnostic lists live in diagnostics/<lane>.json.
  const lean = results.lanes.map(({ diagnostics, ...lane }) => ({
    ...lane,
    diagnosticCount: diagnostics.length,
  }));
  writeFileSync(
    join(out, "results.json"),
    `${JSON.stringify({ ...results, lanes: lean, summary }, null, 1)}\n`,
  );
  writeFileSync(join(out, "summary.md"), renderPairSummary(results, summary));
  if (summary.unstable.length > 0) {
    throw new Error(`diagnostics changed between runs of one lane: ${summary.unstable.join(", ")}`);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
