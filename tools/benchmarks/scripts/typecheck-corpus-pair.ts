/**
 * Real-corpus type-check pair: misskey and n8n checked by tsc and tsrs
 * (TypeScript 7.1 hosts) through the base and head Vize mappers, with Vue's
 * mapper as the reference. vue-tsc (TypeScript 6, the benchmark workspace's
 * pin) checks the same tsconfig once per corpus as a second reference.
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
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { runOrder, summarizeProbe } from "./typecheck-corpus-pair-parse.ts";
import {
  type PairResults,
  type ReferenceResult,
  type SingleCheckerResult,
  renderPairSummary,
  singleCheckerResult,
  summarizePair,
} from "./typecheck-corpus-pair-report.ts";
import { GNU_TIME, type HostRun, hostArgs, hostRun } from "./typecheck-corpus-pair-host.ts";
import {
  diffBatchMapper,
  hostCorsaPath,
  renderBatchDiff,
  runBatchCheck,
} from "./typecheck-corpus-pair-batch.ts";
import {
  assertProjectionBudget,
  assertProbeDiagnosticIdentity,
  checkerBusyMs,
  checkMsByFile,
  probeBytesByFile,
  projectionReport,
  renderProjection,
} from "./typecheck-corpus-pair-projection.ts";
import { familyShapes, renderReplyShape } from "./typecheck-reply-shape.ts";
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
const VUE_TSC_DIR = join(scriptsDir, "node_modules", "vue-tsc");

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
export function prepareProject(dir: string, corpus: Corpus, hosts: MapperHosts): void {
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

function packageVersion(dir: string, name: string): string {
  return (
    JSON.parse(readFileSync(join(dir, "node_modules", name, "package.json"), "utf8")) as {
      version: string;
    }
  ).version;
}

/** vue-tsc and the TypeScript it resolves; a missing install is an error. */
export function resolveVueTsc(): { bin: string; vueTsc: string; typescript: string } {
  const dir = realpathSync(VUE_TSC_DIR);
  const manifest = JSON.parse(readFileSync(join(dir, "package.json"), "utf8")) as {
    version: string;
    bin: { "vue-tsc": string };
  };
  const typescript = createRequire(join(dir, "package.json")).resolve("typescript/package.json");
  return {
    bin: join(dir, manifest.bin["vue-tsc"]),
    vueTsc: manifest.version,
    typescript: (JSON.parse(readFileSync(typescript, "utf8")) as { version: string }).version,
  };
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
      // Also diff the head build's `vize check` against its mapper under tsc.
      batch: { type: "boolean", default: false },
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
  const vueTsc = resolveVueTsc();
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
    const tsconfig = probeTsconfig(mapperOf(lane.variant));
    const run = hostRun(
      [
        bin(lane.host),
        ...hostArgs(
          lane.host,
          cwd(lane),
          tsconfig,
          // Per-SFC check time for the projection report; tsrs writes no trace.
          lane.host.id === "tsc" ? ["--generateTrace", join(work, "traces", key)] : [],
        ),
      ],
      cwd(lane),
      tsconfig,
      join(work, "time", `${key}.probe.time`),
      {
        VIZE_MAPPER_PROBE_RECORD: record,
        // Vize's text per SFC, once per build: the tsrs host receives the same bytes.
        ...(lane.host.id === "tsc" && lane.variant.mapper === "vize"
          ? {
              VIZE_MAPPER_PROBE_DUMP: join(out, "generated", key),
              VIZE_MAPPER_PROBE_ROOT: cwd(lane),
            }
          : {}),
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
  assertProbeDiagnosticIdentity(probes, CORPORA, MAPPER_HOSTS);

  const projections = CORPORA.map((corpus) => {
    const side = (variant: Variant) => {
      const lane = { corpus, host: MAPPER_HOSTS.find((host) => host.id === "tsc")!, variant };
      const key = laneKey(lane);
      return {
        bytes: probeBytesByFile(
          readFileSync(join(work, "time", `${key}.probe.jsonl`), "utf8"),
          cwd(lane),
        ),
        check: checkMsByFile(join(work, "traces", key), cwd(lane)),
        busy: checkerBusyMs(join(work, "traces", key)),
      };
    };
    const [base, head, vue] = VARIANTS.map(side);
    return projectionReport(corpus.id, { base: base!, head: head!, vue: vue! });
  });
  writeFileSync(join(out, "projection.json"), `${JSON.stringify(projections, null, 1)}\n`);
  writeFileSync(
    join(out, "projection.md"),
    projections.map((report) => renderProjection(report)).join("\n"),
  );
  assertProjectionBudget(projections);
  const shapes = CORPORA.map((corpus) => {
    const dir = (role: string) => join(out, "generated", `${corpus.id}-tsc-vize-${role}`);
    return renderReplyShape(corpus.id, familyShapes(dir("base")), familyShapes(dir("head")));
  });
  writeFileSync(join(out, "reply-shape.md"), `### Reply shape\n\n${shapes.join("\n")}`);

  const timed = new Map<string, HostRun[]>();
  for (let run = 0; run < runs; run++) {
    for (const corpus of CORPORA) {
      for (const host of MAPPER_HOSTS) {
        for (const variant of runOrder(VARIANTS, run)) {
          const lane = { corpus, host, variant };
          const key = laneKey(lane);
          const tsconfig = mapperTsconfigName(mapperOf(variant));
          const result = hostRun(
            [bin(host), ...hostArgs(host, cwd(lane), tsconfig)],
            cwd(lane),
            tsconfig,
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

  // Untimed: one checker splits no files, so its counts move only with the text's meaning.
  const tsc = MAPPER_HOSTS.find((host) => host.id === "tsc")!;
  const singleChecker = CORPORA.flatMap((corpus) =>
    VARIANTS.filter((variant) => variant.mapper === "vize").map((variant): SingleCheckerResult => {
      const lane = { corpus, host: tsc, variant };
      const key = laneKey(lane);
      const tsconfig = mapperTsconfigName(mapperOf(variant));
      const run = hostRun(
        [bin(tsc), ...hostArgs(tsc, cwd(lane), tsconfig, ["--checkers", "1"])],
        cwd(lane),
        tsconfig,
        join(work, "time", `${key}.single.time`),
      );
      const result = singleCheckerResult(corpus.id, tsc.id, variant.id, run);
      console.log(`${key}: one checker, ${result.types} types, ${result.instantiations} inst`);
      return result;
    }),
  );

  // One untimed vue-tsc run per corpus on the tsconfig both mapper configs extend.
  const references: ReferenceResult[] = CORPORA.map((corpus) => {
    const dir = projects(corpus, "base");
    const run = hostRun(
      [
        process.execPath,
        "--max-old-space-size=16384",
        vueTsc.bin,
        "--noEmit",
        "--extendedDiagnostics",
        "--pretty",
        "false",
        "-p",
        join(dir, "tsconfig.json"),
      ],
      dir,
      "tsconfig.json",
      join(work, "time", `${corpus.id}-vue-tsc.time`),
    );
    writeFileSync(
      join(out, "diagnostics", `${corpus.id}-vue-tsc.json`),
      `${JSON.stringify(run.diagnostics, null, 1)}\n`,
    );
    console.log(
      `${corpus.id}-vue-tsc: cpu ${(run.cpuMs / 1000).toFixed(2)}s, ${run.diagnostics.length} diagnostics`,
    );
    return {
      corpus: corpus.id,
      tool: "vue-tsc",
      cpuMs: run.cpuMs,
      maxRssKb: run.time.maxRssKb,
      fingerprint: run.fingerprint,
      diagnostics: run.diagnostics,
    };
  });

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
        "vue-tsc": vueTsc.vueTsc,
        "vue-tsc typescript": vueTsc.typescript,
      },
    },
    references,
    singleChecker,
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
  if (values.batch) {
    const corsa = hostCorsaPath(TYPECHECK_HOSTS_DIR);
    const headBuild = resolve(values.head);
    const diffs = CORPORA.map((corpus) => {
      const project = projects(corpus, "head");
      const mapper = timed.get(`${corpus.id}-tsc-vize-head`)?.[0]?.diagnostics;
      if (!mapper) throw new Error(`${corpus.id}: no tsc mapper run on head`);
      const one = runBatchCheck(headBuild, project, corsa);
      // vize check pins one Corsa checker for determinism (#3905); measure what width changes.
      const checkers = 8;
      const wide = runBatchCheck(headBuild, project, corsa, {
        VIZE_CHECKERS: String(checkers),
      });
      const widened = diffBatchMapper(corpus.id, wide, one);
      return {
        ...diffBatchMapper(corpus.id, one, mapper),
        wide: { checkers, added: widened.batchOnly, removed: widened.mapperOnly },
      };
    });
    writeFileSync(join(out, "batch-vs-mapper.json"), `${JSON.stringify(diffs, null, 1)}\n`);
    writeFileSync(join(out, "batch-vs-mapper.md"), renderBatchDiff(diffs));
  }
  const summary = summarizePair(results);
  // Full diagnostic lists live in diagnostics/<lane>.json.
  const lean = results.lanes.map(({ diagnostics, ...lane }) => ({
    ...lane,
    diagnosticCount: diagnostics.length,
  }));
  const leanReferences = references.map(({ diagnostics, ...reference }) => ({
    ...reference,
    diagnosticCount: diagnostics.length,
  }));
  writeFileSync(
    join(out, "results.json"),
    `${JSON.stringify({ ...results, lanes: lean, references: leanReferences, summary }, null, 1)}\n`,
  );
  writeFileSync(join(out, "summary.md"), renderPairSummary(results, summary));
  if (summary.unstable.length > 0) {
    throw new Error(`diagnostics changed between runs of one lane: ${summary.unstable.join(", ")}`);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
