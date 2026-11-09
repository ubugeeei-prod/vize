/**
 * E2E floors: tsc 7.1 `--runExternalCode --noEmit` wall time and its parts for
 * Vize's mapper, Vue's mapper and vue-tsc, plus lower bounds measured through
 * proxy mappers:
 *
 * - stub: every SFC projects to `export default any` (tsc's fixed cost)
 * - replay: the mapper's recorded responses served instantly (zero mapper wait)
 * - anyab: Vize's projection with contract/listener/slot/event/model typing
 *   rewritten to `any` (script + template expressions only)
 * - anyab-replay: both bounds at once
 *
 *   node run.ts --head <worktree> --runs 3 --work <dir> --out <dir>
 */

import { cpSync, mkdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { CORPORA } from "../typecheck-corpus-pair.ts";
import { GNU_TIME, type HostRun, hostArgs, hostRun } from "../typecheck-corpus-pair-host.ts";
import { median } from "../typecheck-corpus-pair-parse.ts";
import { MAPPER_HOSTS, prepareMapperPackages, resolveMapperHosts, TYPECHECK_HOSTS_DIR } from "../typecheck-mapper-hosts.ts";
import { generateScaleProject, writeProject } from "../scale-project-generator.ts";
import { compactRecording } from "./compact-rec.ts";
import { dropProducerIgnores, dropWhitespaceIgnores } from "./ignore-gaps.ts";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..", "..", "..");
const CORPUS_PACKAGES = resolve(here, "..", "..", "typecheck-corpora", "node_modules");
const tool = (name: string) => join(here, name);

type Corpus = { id: string; prepare: (dir: string) => void };
const corpora: Corpus[] = [
  ...CORPORA.map((corpus) => ({
    id: corpus.id,
    prepare: (dir: string) => {
      for (const entry of corpus.copy) cpSync(join(repoRoot, corpus.source, entry), join(dir, entry), { recursive: true, verbatimSymlinks: true });
      writeFileSync(join(dir, "tsconfig.json"), `${JSON.stringify(corpus.tsconfig, null, 2)}\n`);
    },
  })),
  {
    id: "generated-2500",
    prepare: (dir: string) => writeProject(dir, generateScaleProject({ kind: "vue", sfcs: 2500 }).files, false),
  },
];

function addMapper(project: string, id: string, exec: string[]): string {
  const vize = JSON.parse(readFileSync(join(project, "node_modules/vize/package.json"), "utf8"));
  const dir = join(project, "node_modules", `mapper-${id}`);
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, "package.json"), JSON.stringify({ name: `mapper-${id}`, version: "0.0.0", typescript: { contentMapper: { ...vize.typescript.contentMapper, exec } } }));
  const tsconfig = `tsconfig.${id}-mapper.json`;
  writeFileSync(join(project, tsconfig), JSON.stringify({ extends: "./tsconfig.json", contentMappers: [{ package: `mapper-${id}`, extensions: [".vue"] }] }));
  return tsconfig;
}

const { values } = parseArgs({ options: { head: { type: "string" }, runs: { type: "string", default: "3" }, work: { type: "string" }, out: { type: "string" }, only: { type: "string", default: "" } } });
const runs = Number(values.runs);
const work = resolve(values.work!);
const out = resolve(values.out!);
mkdirSync(join(work, "time"), { recursive: true });
mkdirSync(out, { recursive: true });
const hosts = resolveMapperHosts(TYPECHECK_HOSTS_DIR, join(resolve(values.head!), "npm", "cli"));
const tsc = MAPPER_HOSTS.find((host) => host.id === "tsc")!;
const tsrs = MAPPER_HOSTS.find((host) => host.id === "tsrs")!;
const vueTscDir = realpathSync(join(here, "..", "node_modules", "vue-tsc"));
const vueTscBin = join(vueTscDir, JSON.parse(readFileSync(join(vueTscDir, "package.json"), "utf8")).bin["vue-tsc"]);
const vueTscTs = JSON.parse(readFileSync(createRequire(join(vueTscDir, "package.json")).resolve("typescript/package.json"), "utf8")).version;

type Row = { host: string; lane: string; runs: HostRun[] };
const report: Record<string, Row[]> = {};
const timelines: Record<string, string[]> = {};
const notes: string[] = [];
const only = values.only ? new Set(values.only.split(",")) : null;

for (const corpus of corpora) {
  if (only && !only.has(corpus.id)) continue;
  const dir = join(work, corpus.id);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  corpus.prepare(dir);
  cpSync(CORPUS_PACKAGES, join(dir, "node_modules"), { recursive: true, verbatimSymlinks: true });
  prepareMapperPackages(dir, hosts);
  const vizeExec = ["node", join(dir, "node_modules/vize/bin/vize"), "content-mapper"];
  const vueExec = ["node", join(hosts.vueMapperDir, "server.js")];
  const rec = (id: string) => { const d = join(work, `rec-${corpus.id}-${id}`); mkdirSync(d, { recursive: true }); return d; };
  const cfg = {
    vize: "tsconfig.vize-mapper.json",
    vue: "tsconfig.vue-mapper.json",
    stub: addMapper(dir, "stub", ["node", tool("stub-mapper.ts")]),
    vizeRecord: addMapper(dir, "vize-record", ["node", tool("record-proxy.ts"), ...vizeExec]),
    vueRecord: addMapper(dir, "vue-record", ["node", tool("record-proxy.ts"), ...vueExec]),
    anyabRecord: addMapper(dir, "anyab-record", ["node", tool("record-proxy.ts"), "node", tool("anyablate-proxy.ts"), ...vizeExec]),
    replay: addMapper(dir, "replay", ["node", tool("replay-mapper.ts")]),
    anyab: addMapper(dir, "anyab", ["node", tool("anyablate-proxy.ts"), ...vizeExec]),
    vizeTl: addMapper(dir, "vize-tl", ["node", tool("timeline-proxy.ts"), ...vizeExec]),
    vueTl: addMapper(dir, "vue-tl", ["node", tool("timeline-proxy.ts"), ...vueExec]),
  };
  const run = (key: string, tsconfig: string, env: Record<string, string> = {}): HostRun =>
    hostRun([hosts.tsc, ...hostArgs(tsc, dir, tsconfig)], dir, tsconfig, join(work, "time", `${corpus.id}-${key}.time`), env);
  const runTsrs = (key: string, tsconfig: string, env: Record<string, string> = {}): HostRun =>
    hostRun([hosts.tsrs, ...hostArgs(tsrs, dir, tsconfig)], dir, tsconfig, join(work, "time", `${corpus.id}-tsrs-${key}.time`), env);
  const vueTsc = (): HostRun =>
    hostRun([process.execPath, "--max-old-space-size=16384", vueTscBin, "--noEmit", "--extendedDiagnostics", "--pretty", "false", "-p", join(dir, "tsconfig.json")], dir, "tsconfig.json", join(work, "time", `${corpus.id}-vue-tsc.time`));

  // Untimed: warm the file cache, record responses, capture request timelines.
  run("warm", cfg.vize);
  const recs = { vize: rec("vize"), vue: rec("vue"), anyab: rec("anyab") };
  run("rec-vize", cfg.vizeRecord, { REC_DIR: recs.vize });
  run("rec-vue", cfg.vueRecord, { REC_DIR: recs.vue });
  run("rec-anyab", cfg.anyabRecord, { REC_DIR: recs.anyab });
  const compacted = join(work, `rec-${corpus.id}-vize-compact`);
  notes.push(`${corpus.id}: ${compactRecording(recs.vize, compacted)}`);
  const wsIgnores = join(work, `rec-${corpus.id}-vize-ws-ignores`);
  notes.push(`${corpus.id} whitespace ignores: ${compactRecording(recs.vize, wsIgnores, dropWhitespaceIgnores)}`);
  const noIgnores = join(work, `rec-${corpus.id}-vize-no-ignores`);
  notes.push(`${corpus.id} producer ignores: ${compactRecording(recs.vize, noIgnores, dropProducerIgnores)}`);
  for (const [id, tsconfig] of [["vize", cfg.vizeTl], ["vue", cfg.vueTl]] as const) {
    const log = join(work, `tl-${corpus.id}-${id}.jsonl`);
    run(`tl-${id}`, tsconfig, { TL_LOG: log });
    const { summarize } = await import(tool("timeline-sum.ts"));
    timelines[`${corpus.id}/${id}`] = summarize(readFileSync(log, "utf8"));
  }

  const laneFilter = process.env.FLOORS_LANES ? process.env.FLOORS_LANES.split(",") : null;
  const lanes: [string, string, () => HostRun][] = [
    ["tsc", "vize", () => run("vize", cfg.vize)],
    ["tsc", "vue", () => run("vue", cfg.vue)],
    ["tsc", "stub (fixed cost)", () => run("stub", cfg.stub)],
    ["tsc", "vize replay (zero wait)", () => run("replay-vize", cfg.replay, { REC_DIR: recs.vize })],
    ["tsc", "vize replay compacted", () => run("replay-compact", cfg.replay, { REC_DIR: compacted })],
    ["tsc", "vize replay without whitespace ignores", () => run("replay-ws", cfg.replay, { REC_DIR: wsIgnores })],
    ["tsc", "vize replay without producer ignores (bound)", () => run("replay-noign", cfg.replay, { REC_DIR: noIgnores })],
    ["tsc", "vue replay (zero wait)", () => run("replay-vue", cfg.replay, { REC_DIR: recs.vue })],
    ["tsc", "vize expressions-only", () => run("anyab", cfg.anyab)],
    ["tsc", "vize expressions-only replay", () => run("replay-anyab", cfg.replay, { REC_DIR: recs.anyab })],
    ["tsrs", "vize", () => runTsrs("vize", cfg.vize)],
    ["tsrs", "vue", () => runTsrs("vue", cfg.vue)],
    ["tsrs", "stub (fixed cost)", () => runTsrs("stub", cfg.stub)],
    ["tsrs", "vize replay (zero wait)", () => runTsrs("replay-vize", cfg.replay, { REC_DIR: recs.vize })],
    ["tsrs", "vize replay compacted", () => runTsrs("replay-compact", cfg.replay, { REC_DIR: compacted })],
    ["tsrs", "vize replay without whitespace ignores", () => runTsrs("replay-ws", cfg.replay, { REC_DIR: wsIgnores })],
    ["tsrs", "vize replay without producer ignores (bound)", () => runTsrs("replay-noign", cfg.replay, { REC_DIR: noIgnores })],
    ["tsrs", "vue replay (zero wait)", () => runTsrs("replay-vue", cfg.replay, { REC_DIR: recs.vue })],
    ["tsrs", "vize expressions-only replay", () => runTsrs("replay-anyab", cfg.replay, { REC_DIR: recs.anyab })],
  ];
  if (laneFilter) lanes.splice(0, lanes.length, ...lanes.filter(([, lane]) => laneFilter.some((token) => lane.startsWith(token))));
  const rows: Row[] = lanes.map(([host, lane]) => ({ host, lane, runs: [] }));
  for (let i = 0; i < runs; i++) {
    const order = i % 2 === 0 ? lanes.map((_, k) => k) : lanes.map((_, k) => lanes.length - 1 - k);
    for (const k of order) {
      const result = lanes[k]![2]();
      rows[k]!.runs.push(result);
      console.log(`${corpus.id} ${lanes[k]![0]} ${lanes[k]![1]} run ${i + 1}: wall ${(result.wallMs / 1000).toFixed(2)}s, ${result.diagnostics.length} diagnostics`);
    }
  }
  const vt: Row = { host: "tsc", lane: `vue-tsc (typescript ${vueTscTs})`, runs: [] };
  if (!laneFilter) {
    for (let i = 0; i < Math.min(runs, 3); i++) vt.runs.push(vueTsc());
    rows.push(vt);
  }
  report[corpus.id] = rows;
  writeFileSync(join(out, `${corpus.id}.json`), JSON.stringify(rows.map((row) => ({ host: row.host, lane: row.lane, runs: row.runs.map(({ diagnostics, ...r }) => ({ ...r, diagnosticCount: diagnostics.length })) })), null, 1));
}

const stat = (run: HostRun, pattern: RegExp): number | null => {
  const entry = run.stats.find(([name]) => pattern.test(name));
  return entry ? Number.parseFloat(entry[1]) : null;
};
const med = (row: Row, pick: (run: HostRun) => number | null): string => {
  const xs = row.runs.map(pick).filter((x): x is number => x != null);
  return xs.length ? String(median(xs)) : "–";
};
const fmt = (s: string, digits: number) => (s === "–" ? s : Number(s).toFixed(digits));
const lines = [
  "## E2E floors (`--runExternalCode --noEmit` on tsc 7.1 and tsrs --fast)",
  "",
  `Median of ${runs} runs per lane (vue-tsc: ${Math.min(runs, 3)}), on \`${process.env.VIZE_PAIR_RUNNER_LABEL ?? "local"}\`, GNU time at ${GNU_TIME}. Wall is process start to exit; parts are tsc's \`--extendedDiagnostics\`.`,
  "",
];
for (const [corpus, all] of Object.entries(report)) for (const host of ["tsc", "tsrs"]) {
  const rows = all.filter((row) => row.host === host);
  lines.push(`### ${corpus}, ${host === "tsc" ? "tsc 7.1" : "tsrs --fast"}`, "", "| lane | wall s | Total s | parse s | mapper wait s | bind s | check s | types | instantiations | memory MB | CPU s | diagnostics |", "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
  for (const row of rows) {
    lines.push(`| ${row.lane} | ${fmt(med(row, (r) => r.wallMs / 1000), 2)} | ${fmt(med(row, (r) => stat(r, /^Total time$/)), 3)} | ${fmt(med(row, (r) => stat(r, /^Parse time$/) ?? stat(r, /^I\/O read$/)), 3)} | ${fmt(med(row, (r) => stat(r, /request wait time$/)), 3)} | ${fmt(med(row, (r) => stat(r, /^Bind time$/)), 3)} | ${fmt(med(row, (r) => stat(r, /^Check time$/)), 3)} | ${fmt(med(row, (r) => stat(r, /^Types$/)), 0)} | ${fmt(med(row, (r) => stat(r, /^Instantiations$/)), 0)} | ${fmt(med(row, (r) => (stat(r, /^Memory used$/) ?? 0) / 1024), 0)} | ${fmt(med(row, (r) => r.cpuMs / 1000), 2)} | ${row.runs[0]?.diagnostics.length ?? "–"} |`);
  }
  lines.push("");
}
lines.push("### Compacted replies", "", ...notes.map((note) => `- ${note}`), "");
lines.push("### Request timelines (proxy-observed, ms from mapper start)", "");
for (const [key, summary] of Object.entries(timelines)) lines.push(`- ${key}: ${summary.join("; ")}`);
writeFileSync(join(out, "summary.md"), `${lines.join("\n")}\n`);
console.log(lines.join("\n"));
