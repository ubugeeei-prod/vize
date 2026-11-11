/** Markdown of the cold wall breakdown: the 500 ms target first, then every lane per corpus. */

import { writeFileSync } from "node:fs";
import { join } from "node:path";

import {
  type ColdLane,
  type ColdResults,
  type ColdSummary,
  TARGET_MS,
  dominantPart,
  hostLabel,
  summarizeCold,
} from "./typecheck-corpus-pair-cold.ts";
import type { FloorRecordings } from "./typecheck-corpus-pair-floors.ts";
import { PAIR_HOSTS } from "./typecheck-corpus-pair-host.ts";
import { median } from "./typecheck-corpus-pair-parse.ts";
import type { LaneSummary } from "./typecheck-corpus-pair-report.ts";

const ms = (value: number | null) => (value == null ? "n/a" : Math.round(value).toString());
const mb = (value: number | null) => (value == null ? "n/a" : value.toFixed(0));
const target = (wallMs: number) =>
  `${wallMs <= TARGET_MS ? "≤" : "+"}${Math.round(Math.abs(wallMs - TARGET_MS))}`;
const parseOnly = (lane: ColdSummary) =>
  lane.parseMs == null ? null : Math.max(0, lane.parseMs - (lane.waitMs ?? 0));

const VARIANT_LABELS: Record<string, string> = {
  "vize-head": "Vize head",
  "vize-base": "Vize base",
  vue: "Vue mapper",
  "floor-absolute": "floor: absolute (export default {} as any)",
  "floor-expressions": "floor: expressions-only, zero wait",
  "floor-replay": "floor: Vize head, zero wait",
};

const VARIANT_ORDER = Object.keys(VARIANT_LABELS);
/** The release the target names first, then the pinned host it is measured against. */
const TARGET_HOSTS = ["tsrs-0.23-fast", "tsrs"];
const laneOrder = (lane: { host: string; variant: string }) =>
  PAIR_HOSTS.findIndex((host) => host.id === lane.host) * 100 + VARIANT_ORDER.indexOf(lane.variant);

export function renderCold(results: ColdResults): string {
  const summaries = results.lanes.map(summarizeCold).sort((a, b) => laneOrder(a) - laneOrder(b));
  const find = (corpus: string, host: string, variant: string) =>
    summaries.find(
      (lane) => lane.corpus === corpus && lane.host === host && lane.variant === variant,
    );
  const lines = [
    "## Cold check wall",
    "",
    `Target: \`tsrs --fast\` + Vize ≤ ${TARGET_MS} ms wall per corpus. Median of ${results.runs} run(s) per lane on \`${results.runner}\`. Every timed run is a fresh host process with \`incremental: false\`, XDG_CACHE_HOME and TMPDIR in a new empty directory, NODE_DISABLE_COMPILE_CACHE=1 and VIZE_CONFIG_CACHE=0; "cache files" counts what a run wrote there. The OS page cache stays warm.`,
    "",
    "Wall is spawn to exit of the host's process tree. Config, parse, mapper wait, bind, check and Total are the host's `--extendedDiagnostics`; the mapper wait lies inside parse, so \"parse − wait\" is the parse work the wait does not cover. Outside Total = wall − Total: the npm launcher, process start, the report and teardown (waiting for the mapper to exit included). Tree RSS is the peak summed VmRSS of every process in the run (5 ms /proc samples); host and mapper RSS are each process's VmHWM; largest is GNU time %M. Reply bytes, spans and directives are the probe run's.",
    "",
    "### Target lanes: tsrs --fast + Vize head",
    "",
    "| corpus | host | SFC | wall ms | vs 500 ms | outside Total | mapper wait | parse − wait | check | tree RSS MB | dominant part | absolute floor ms | expressions-only ms | zero-wait ms |",
    "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: |",
  ];
  for (const [corpus, host] of results.corpora.flatMap((entry) =>
    TARGET_HOSTS.map((id) => [entry, id] as const),
  )) {
    const lane = find(corpus.id, host, "vize-head");
    if (!lane) continue;
    const dominant = dominantPart(lane);
    const floor = (variant: string) => ms(find(corpus.id, host, variant)?.wallMs ?? null);
    lines.push(
      `| ${corpus.label} | ${hostLabel(host)} | ${corpus.sfcs} | ${ms(lane.wallMs)} | ${target(lane.wallMs)} | ${ms(lane.outsideTotalMs)} | ${ms(lane.waitMs)} | ${ms(parseOnly(lane))} | ${ms(lane.checkMs)} | ${mb(lane.treeMb)} | ${dominant ? `${dominant.part} (${ms(dominant.ms)} ms)` : "n/a"} | ${floor("floor-absolute")} | ${floor("floor-expressions")} | ${floor("floor-replay")} |`,
    );
  }
  lines.push("", "### Process floor (`<host> --version`)", "");
  for (const entry of results.startup)
    lines.push(`- ${hostLabel(entry.host)}: ${ms(median(entry.wallMs))} ms`);
  lines.push("");
  for (const corpus of results.corpora) {
    lines.push(
      `### ${corpus.label} (${corpus.sfcs} SFC)`,
      "",
      "| host | lane | wall ms | vs 500 ms | outside Total | config | parse | mapper wait | parse − wait | bind | check | Total | CPU s | tree RSS MB | host RSS MB | mapper RSS MB (procs) | largest MB | reply MB | spans | directives | cache files |",
      `| --- | --- |${" ---: |".repeat(19)}`,
    );
    for (const lane of summaries.filter((entry) => entry.corpus === corpus.id)) {
      const reply = results.replies[`${lane.corpus}-${lane.host}-${lane.variant}`];
      lines.push(
        `| ${hostLabel(lane.host)} | ${VARIANT_LABELS[lane.variant] ?? lane.variant} | ${ms(lane.wallMs)} | ${target(lane.wallMs)} | ${ms(lane.outsideTotalMs)} | ${ms(lane.configMs)} | ${ms(lane.parseMs)} | ${ms(lane.waitMs)} | ${ms(parseOnly(lane))} | ${ms(lane.bindMs)} | ${ms(lane.checkMs)} | ${ms(lane.totalMs)} | ${lane.cpuS.toFixed(2)} | ${mb(lane.treeMb)} | ${mb(lane.hostMb)} | ${mb(lane.mapperMb)} (${lane.mappers ?? "n/a"}) | ${mb(lane.largestMb)} | ${reply ? (reply.replyBytes / 1048576).toFixed(1) : "n/a"} | ${reply ? reply.spans.toLocaleString("en-US") : "n/a"} | ${reply ? reply.directives.toLocaleString("en-US") : "n/a"} | ${lane.cacheFiles} |`,
      );
    }
    lines.push("");
  }
  if (results.notes.length > 0) lines.push(...results.notes.map((note) => `- ${note}`), "");
  return `${lines.join("\n")}\n`;
}

/** Floor fidelity and stability notes, then cold.json (runs without diagnostics) and cold.md. */
export function writeCold(
  out: string,
  input: Omit<ColdResults, "replies" | "notes" | "corpora" | "lanes"> & {
    corpora: ReadonlyArray<{ id: string; label: string; sfcs: number }>;
    lanes: ColdLane[];
    summary: LaneSummary[];
    recordings: ReadonlyMap<string, FloorRecordings>;
  },
): void {
  const { summary, recordings, ...rest } = input;
  const replies = Object.fromEntries(
    summary.map(({ corpus, host, variant, replyBytes, spans, directives }) => [
      `${corpus}-${host}-${variant}`,
      { replyBytes, spans, directives },
    ]),
  );
  const notes: string[] = [];
  for (const [corpus, recorded] of recordings) {
    const { files, bytesBefore, bytesAfter, counts } = recorded.ablation;
    notes.push(
      `${corpus}: expressions-only rewrote ${JSON.stringify(counts)} over ${files} SFCs, reply bytes ${bytesBefore} → ${bytesAfter}`,
    );
  }
  const lanes = input.lanes.map((lane) => ({
    ...lane,
    runs: lane.runs.map(
      ({
        diagnostics: _diagnostics,
        ...run
      }: ColdLane["runs"][number] & { diagnostics?: unknown }) => run,
    ),
  }));
  for (const lane of lanes.filter((entry) => entry.variant.startsWith("floor-"))) {
    const prints = new Set(lane.runs.map((run) => run.fingerprint));
    if (prints.size > 1)
      notes.push(
        `${lane.corpus} / ${lane.host} / ${lane.variant}: diagnostics differ between runs`,
      );
    if (lane.variant !== "floor-replay") continue;
    const live = lanes.find(
      (entry) =>
        entry.corpus === lane.corpus && entry.host === lane.host && entry.variant === "vize-head",
    );
    const same = live?.runs[0]?.fingerprint === lane.runs[0]?.fingerprint;
    notes.push(
      `${lane.corpus} / ${hostLabel(lane.host)}: zero-wait replay diagnostics ${same ? "IDENTICAL to" : "DIFFER from"} Vize head`,
    );
  }
  const results: ColdResults = {
    ...rest,
    corpora: input.corpora.map(({ id, label, sfcs }) => ({ id, label, sfcs })),
    lanes,
    replies,
    notes,
  };
  writeFileSync(join(out, "cold.json"), `${JSON.stringify(results, null, 1)}\n`);
  writeFileSync(join(out, "cold.md"), renderCold(results));
}
