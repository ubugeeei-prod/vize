/**
 * Whole-process-tree CPU profiles of the type-check lanes on the corpus-pair
 * corpora (Linux, perf): tsc (tsgo) + Vize mapper, tsrs + Vize mapper and
 * `vize check`, for a head build and optionally a base build.
 *
 *   node typecheck-profile.ts --head <worktree> --head-sha <sha> \
 *     [--base <worktree> --base-sha <sha>] --work <dir> --out <dir> \
 *     [--corpora misskey,n8n] [--lanes tsc,tsrs,check] [--perf <perf>] \
 *     [--frequency 999] [--flamegraph <FlameGraph checkout>]
 *   node typecheck-profile.ts --print-submodules [--corpora ...]
 *
 * Per lane and side: one untimed run (with tsgo `--pprofDir` on the tsc
 * lane), one `perf stat` run (counters of the whole tree) and one
 * `perf record` run whose `perf script` is folded by typecheck-profile-fold
 * into role/DSO/phase/function tables, folded stacks and a flame graph.
 * Every run starts from an empty XDG cache. Build each worktree's native
 * addon with frame pointers and symbols (see typecheck-profile.yml).
 */

import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { CORPORA, prepareProject } from "./typecheck-corpus-pair.ts";
import {
  type MapperHosts,
  TYPECHECK_HOSTS_DIR,
  resolveMapperHosts,
} from "./typecheck-mapper-hosts.ts";
import { LANES, type LaneContext, run } from "./typecheck-profile-lanes.ts";
import {
  PERF_MAP_DIR,
  flamegraph,
  foldPerfData,
  parseStatCsv,
  perfMaps,
  statEvents,
  withoutPerfNoise,
  writeGzip,
} from "./typecheck-profile-perf.ts";
import { readPprofTop } from "./typecheck-profile-pprof.ts";
import { type LaneResult, renderLane, renderOverview } from "./typecheck-profile-report.ts";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptsDir, "..", "..", "..");
type Side = "base" | "head";

/** The submodules holding the selected corpora: the longest .gitmodules path prefixing each source. */
export function corpusSubmodules(
  corpora: ReadonlyArray<{ id: string; source: string }>,
  gitmodules = readFileSync(join(repoRoot, ".gitmodules"), "utf8"),
): string[] {
  const paths = [...gitmodules.matchAll(/^\s*path\s*=\s*(\S+)\s*$/gmu)].map((match) => match[1]!);
  return corpora.map((corpus) => {
    const path = paths
      .filter(
        (candidate) => corpus.source === candidate || corpus.source.startsWith(`${candidate}/`),
      )
      .sort((a, b) => b.length - a.length)[0];
    if (!path) throw new Error(`${corpus.id}: no submodule holds ${corpus.source}`);
    return path;
  });
}

/** The entries a comma list names, in their own order; an unknown id is an error. */
export function selectIds<T extends { id: string }>(
  all: readonly T[],
  list: string | undefined,
  what: string,
): T[] {
  if (!list?.trim()) return [...all];
  const ids = list
    .split(",")
    .map((id) => id.trim())
    .filter(Boolean);
  for (const id of ids) {
    if (!all.some((item) => item.id === id)) throw new Error(`unknown ${what}: ${id}`);
  }
  return all.filter((item) => ids.includes(item.id));
}

type ProfileOptions = {
  perf: string;
  events: string[];
  frequency: number;
  event: string;
  dwarfStack: string;
  flamegraph: string | undefined;
  work: string;
  out: string;
};

async function profileLane(
  options: ProfileOptions,
  corpus: string,
  laneId: string,
  side: Side,
  sha: string,
  context: LaneContext,
): Promise<LaneResult> {
  const { perf, work, out } = options;
  const lane = LANES[laneId]!;
  const key = `${corpus}-${laneId}-${side}`;
  const cache = join(work, "cache", key);
  // vize check keeps tsgo build info in the user cache: every run starts empty.
  const env = () => {
    rmSync(cache, { recursive: true, force: true });
    mkdirSync(cache, { recursive: true });
    return { XDG_CACHE_HOME: cache };
  };
  const notes: string[] = [];
  const pprofDir = join(work, "pprof", key);
  rmSync(pprofDir, { recursive: true, force: true });
  mkdirSync(pprofDir, { recursive: true });
  const warm = run(
    lane.command(context, lane.pprof ? ["--pprofDir", pprofDir] : []),
    context.project,
    env(),
  );
  const diagnostics = lane.diagnostics(warm, context.project);
  let pprof = null;
  if (lane.pprof) {
    const file = readdirSync(pprofDir).find((name) => /cpu/iu.test(name));
    if (file) {
      pprof = readPprofTop(join(pprofDir, file));
      writeFileSync(join(out, "pprof", `${key}.pb.gz`), readFileSync(join(pprofDir, file)));
    } else notes.push(`tsgo wrote no CPU profile to --pprofDir (status ${warm.status}).`);
  }

  const statFile = join(out, "stat", `${key}.csv`);
  const stat = withoutPerfNoise(
    run(
      [
        perf,
        "stat",
        "-x,",
        "-o",
        statFile,
        "-e",
        options.events.join(","),
        "--",
        ...lane.command(context, []),
      ],
      context.project,
      env(),
    ),
  );
  const counters = parseStatCsv(readFileSync(statFile, "utf8"));
  const statDiagnostics = lane.diagnostics(stat, context.project);
  if (statDiagnostics !== diagnostics) {
    notes.push(`diagnostics moved: ${diagnostics} untimed, ${statDiagnostics} under perf stat.`);
  }

  const data = join(work, `${key}.perf.data`);
  const callGraph = lane.callGraph === "dwarf" ? `dwarf,${options.dwarfStack}` : "fp";
  const before = perfMaps();
  const record = run(
    [
      perf,
      "record",
      "-e",
      options.event,
      "-F",
      String(options.frequency),
      "--call-graph",
      callGraph,
      "-m",
      "16M",
      "-o",
      data,
      "--",
      ...lane.command(context, []),
    ],
    context.project,
    {
      ...env(),
      NODE_OPTIONS: "--perf-basic-prof-only-functions --interpreted-frames-native-stack",
    },
  );
  writeFileSync(join(out, "logs", `${key}.record.stderr.txt`), record.stderr);
  for (const line of record.stderr.split("\n")) {
    if (/lost|throttl|overload/iu.test(line)) notes.push(`perf record: ${line.trim()}`);
  }
  const recordDiagnostics = lane.diagnostics(withoutPerfNoise(record), context.project);
  if (recordDiagnostics !== diagnostics) {
    notes.push(`diagnostics moved under perf record: ${recordDiagnostics} vs ${diagnostics}.`);
  }

  const { folder, stderr } = await foldPerfData(perf, data, lane.shape);
  writeFileSync(join(out, "logs", `${key}.script.stderr.txt`), stderr);
  for (const file of perfMaps())
    if (!before.has(file)) rmSync(join(PERF_MAP_DIR, file), { force: true });
  rmSync(data, { force: true });
  rmSync(`${data}.old`, { force: true });
  const profile = folder.finish();
  const folded = join(work, "folded", `${key}.folded`);
  mkdirSync(dirname(folded), { recursive: true });
  writeFileSync(folded, `${[...folder.folded()].join("\n")}\n`);
  await writeGzip(folder.folded(), join(out, "folded", `${key}.folded.gz`));
  if (options.flamegraph) {
    flamegraph(
      options.flamegraph,
      folded,
      join(out, "flamegraphs", `${key}.svg`),
      `${corpus} ${lane.label} (${side} ${sha.slice(0, 10)})`,
    );
  }

  const result: LaneResult = {
    corpus,
    lane: laneId,
    label: lane.label,
    side,
    sha,
    callGraph,
    frequency: options.frequency,
    wallMs: stat.wallMs,
    diagnostics,
    counters,
    hostStats: lane.stats ? lane.stats(stat, context.project) : [],
    profile,
    pprof,
    notes,
  };
  writeFileSync(join(out, "profiles", `${key}.json`), `${JSON.stringify(result)}\n`);
  return result;
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      head: { type: "string" },
      "head-sha": { type: "string" },
      base: { type: "string" },
      "base-sha": { type: "string" },
      work: { type: "string" },
      out: { type: "string" },
      corpora: { type: "string" },
      lanes: { type: "string", default: "tsc,tsrs,check" },
      perf: { type: "string", default: "perf" },
      frequency: { type: "string", default: "999" },
      event: { type: "string", default: "cpu-clock" },
      "dwarf-stack": { type: "string", default: "16384" },
      flamegraph: { type: "string" },
      "print-submodules": { type: "boolean", default: false },
    },
  });
  const corpora = selectIds(CORPORA, values.corpora, "corpus");
  if (values["print-submodules"]) {
    console.log(corpusSubmodules(corpora).join(" "));
    return;
  }
  if (!values.head || !values.work || !values.out) {
    throw new Error("--head, --work and --out are required");
  }
  const sides: Array<{ side: Side; build: string; sha: string }> = [];
  if (values.base) {
    sides.push({ side: "base", build: resolve(values.base), sha: values["base-sha"] ?? "" });
  }
  sides.push({ side: "head", build: resolve(values.head), sha: values["head-sha"] ?? "" });
  const laneIds = values.lanes
    .split(",")
    .map((id) => id.trim())
    .filter(Boolean);
  for (const id of laneIds) {
    if (!LANES[id])
      throw new Error(`unknown lane: ${id} (known: ${Object.keys(LANES).join(", ")})`);
  }
  const work = resolve(values.work);
  const out = resolve(values.out);
  for (const dir of ["profiles", "folded", "flamegraphs", "pprof", "stat", "logs"]) {
    mkdirSync(join(out, dir), { recursive: true });
  }
  mkdirSync(work, { recursive: true });
  const perf = values.perf;
  const events = statEvents(perf);
  const perfVersion = run([perf, "--version"], work).stdout.trim();
  console.log(`${perfVersion}; stat events: ${events.join(",")}`);
  const hosts = Object.fromEntries(
    sides.map(({ side, build }) => [
      side,
      resolveMapperHosts(TYPECHECK_HOSTS_DIR, join(build, "npm", "cli")),
    ]),
  ) as Record<Side, MapperHosts>;
  const options: ProfileOptions = {
    perf,
    events,
    frequency: Number(values.frequency),
    event: values.event,
    dwarfStack: values["dwarf-stack"],
    flamegraph: values.flamegraph,
    work,
    out,
  };

  const results: LaneResult[] = [];
  const failures: string[] = [];
  for (const corpus of corpora) {
    const projects = {} as Record<Side, string>;
    for (const { side } of sides) {
      projects[side] = join(work, `${corpus.id}-${side}`);
      prepareProject(projects[side], corpus, hosts[side]);
    }
    for (const [laneIndex, laneId] of laneIds.entries()) {
      // Alternate which build goes first so drift over the job favors neither side.
      const order = laneIndex % 2 === 0 ? sides : [...sides].reverse();
      for (const { side, build, sha } of order) {
        const key = `${corpus.id}-${laneId}-${side}`;
        try {
          const context = { build, project: projects[side], hosts: hosts[side] };
          const result = await profileLane(options, corpus.id, laneId, side, sha, context);
          results.push(result);
          const { profile } = result;
          console.log(
            `${key}: ${profile.samples} samples, ${(profile.period / 1e9).toFixed(2)} s sampled, wall ${(result.wallMs / 1000).toFixed(2)} s, ${result.diagnostics} diagnostics`,
          );
        } catch (error) {
          failures.push(`${key}: ${(error as Error).message.slice(0, 4000)}`);
          console.error(`${key} failed: ${(error as Error).stack}`);
        }
      }
      const folded = (side: Side) => join(work, "folded", `${corpus.id}-${laneId}-${side}.folded`);
      if (values.flamegraph && existsSync(folded("base")) && existsSync(folded("head"))) {
        flamegraph(
          values.flamegraph,
          folded("head"),
          join(out, "flamegraphs", `${corpus.id}-${laneId}-diff.svg`),
          `${corpus.id} ${LANES[laneId]!.label}: base -> head (red grew)`,
          folded("base"),
        );
      }
    }
  }

  const head = sides.find((side) => side.side === "head")!;
  const base = sides.find((side) => side.side === "base");
  const baseOf = (result: LaneResult) =>
    results.find(
      (r) => r.side === "base" && r.corpus === result.corpus && r.lane === result.lane,
    ) ?? null;
  const lanes = results
    .filter((result) => result.side === "head")
    .map((result) => renderLane(result, baseOf(result)));
  const failed = failures.length
    ? ["### Failed lanes", "", ...failures.map((failure) => `- ${failure.split("\n")[0]}`), ""]
    : [];
  const header = [
    `## Type-check profile: head \`${head.sha.slice(0, 12)}\`${base ? ` vs base \`${base.sha.slice(0, 12)}\`` : ""}`,
    "",
    `${perfVersion}; \`${options.event}\` sampled at ${options.frequency} Hz over each lane's whole process tree (perf record follows every child). One profiled run per lane and side: moves of a few percent in one function are within run-to-run noise.`,
    "",
    renderOverview(results),
  ];
  writeFileSync(join(out, "summary.md"), [...header, ...failed, ...lanes].join("\n"));
  writeFileSync(
    join(out, "results.json"),
    `${JSON.stringify({ perf: perfVersion, statEvents: events, sides, lanes: laneIds, corpora: corpora.map((c) => c.id), failures }, null, 1)}\n`,
  );
  if (failures.length)
    throw new Error(`${failures.length} lane runs failed:\n${failures.join("\n")}`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
