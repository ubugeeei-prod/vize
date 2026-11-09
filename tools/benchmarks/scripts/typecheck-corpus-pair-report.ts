/**
 * Medians, diagnostic identity, agreement with the vue-tsc reference and the
 * Markdown table of the real-corpus type-check pair.
 */

import {
  type Diagnostic,
  type GnuTime,
  type ReplyShape,
  type StatEntry,
  diffDiagnostics,
  median,
  statValue,
} from "./typecheck-corpus-pair-parse.ts";

export type LaneRun = {
  wallMs: number;
  cpuMs: number;
  time: GnuTime;
  status: number | null;
  stats: StatEntry[];
  fingerprint: string;
};

export type LaneResult = {
  corpus: string;
  corpusLabel: string;
  sfcs: number;
  host: string;
  variant: "vize-base" | "vize-head" | "vue";
  probe: {
    files: number;
    transforms: number;
    processes: number;
    bytes: number;
    fingerprint: string;
  } & Partial<ReplyShape>;
  diagnostics: Diagnostic[];
  runs: LaneRun[];
};

/** One untimed run of a reference checker on a corpus's own tsconfig. */
export type ReferenceResult = {
  corpus: string;
  tool: "vue-tsc";
  cpuMs: number;
  maxRssKb: number;
  fingerprint: string;
  diagnostics: Diagnostic[];
};

export type PairResults = {
  schemaVersion: 1;
  base: { sha: string };
  head: { sha: string };
  runs: number;
  measurement: Record<string, unknown> & { runner: string; packages: Record<string, string> };
  lanes: LaneResult[];
  references: ReferenceResult[];
};

export type LaneSummary = {
  corpus: string;
  host: string;
  variant: LaneResult["variant"];
  cpuS: number;
  cpuMinS: number;
  wallS: number;
  maxRssMb: number;
  instantiations: number;
  types: number;
  checkS: number;
  generatedBytes: number;
  replyBytes: number;
  spans: number;
  directives: number;
  ignoreGaps: number;
  diagnostics: number;
  fingerprint: string;
};

export type Identity = {
  corpus: string;
  host: string;
  status: "IDENTICAL" | "CHANGED";
  base: { count: number; fingerprint: string };
  head: { count: number; fingerprint: string };
  added: Diagnostic[];
  removed: Diagnostic[];
};

/** A lane's diagnostics against the reference's: shared, lane-only, reference-only. */
export type Agreement = {
  corpus: string;
  host: string;
  variant: LaneResult["variant"];
  reference: ReferenceResult["tool"];
  shared: number;
  laneOnly: number;
  referenceOnly: number;
};

export type PairSummary = {
  lanes: LaneSummary[];
  identity: Identity[];
  agreement: Agreement[];
  unstable: string[];
};

const laneName = (lane: { corpus: string; host: string; variant: string }) =>
  `${lane.corpus}-${lane.host}-${lane.variant}`;

export function summarizeLane(lane: LaneResult): LaneSummary {
  const of = (pick: (run: LaneRun) => number) => median(lane.runs.map(pick));
  return {
    corpus: lane.corpus,
    host: lane.host,
    variant: lane.variant,
    cpuS: of((run) => run.cpuMs) / 1000,
    cpuMinS: Math.min(...lane.runs.map((run) => run.cpuMs)) / 1000,
    wallS: of((run) => run.wallMs) / 1000,
    maxRssMb: of((run) => run.time.maxRssKb) / 1024,
    instantiations: of((run) => statValue(run.stats, "Instantiations", "count")),
    types: of((run) => statValue(run.stats, "Types", "count")),
    checkS: of((run) => statValue(run.stats, "Check time", "seconds")),
    generatedBytes: lane.probe.bytes,
    replyBytes: lane.probe.replyBytes ?? 0,
    spans: lane.probe.spans ?? 0,
    directives: lane.probe.directives ?? 0,
    ignoreGaps: lane.probe.ignoreGaps ?? 0,
    diagnostics: lane.diagnostics.length,
    fingerprint: lane.probe.fingerprint,
  };
}

/** Every run of a lane, probe included, must report the same diagnostics. */
export function unstableLanes(lanes: LaneResult[]): string[] {
  return lanes
    .filter((lane) => lane.runs.some((run) => run.fingerprint !== lane.probe.fingerprint))
    .map(laneName);
}

export function summarizePair(results: PairResults): PairSummary {
  const lanes = results.lanes.map(summarizeLane);
  const find = (corpus: string, host: string, variant: LaneResult["variant"]) => {
    const lane = results.lanes.find(
      (entry) => entry.corpus === corpus && entry.host === host && entry.variant === variant,
    );
    if (!lane) throw new Error(`missing lane ${corpus}-${host}-${variant}`);
    return lane;
  };
  const pairs = [...new Set(results.lanes.map((lane) => `${lane.corpus}\0${lane.host}`))];
  const identity = pairs.map((pair): Identity => {
    const [corpus, host] = pair.split("\0") as [string, string];
    const base = find(corpus, host, "vize-base");
    const head = find(corpus, host, "vize-head");
    const { added, removed } = diffDiagnostics(base.diagnostics, head.diagnostics);
    return {
      corpus,
      host,
      status: added.length === 0 && removed.length === 0 ? "IDENTICAL" : "CHANGED",
      base: { count: base.diagnostics.length, fingerprint: base.probe.fingerprint },
      head: { count: head.diagnostics.length, fingerprint: head.probe.fingerprint },
      added,
      removed,
    };
  });
  const agreement = results.references.flatMap((reference) =>
    results.lanes
      .filter((lane) => lane.corpus === reference.corpus && lane.variant !== "vize-base")
      .map((lane): Agreement => {
        const { added, removed } = diffDiagnostics(reference.diagnostics, lane.diagnostics);
        return {
          corpus: lane.corpus,
          host: lane.host,
          variant: lane.variant,
          reference: reference.tool,
          shared: lane.diagnostics.length - added.length,
          laneOnly: added.length,
          referenceOnly: removed.length,
        };
      }),
  );
  return { lanes, identity, agreement, unstable: unstableLanes(results.lanes) };
}

const integer = (value: number) => Math.round(value).toLocaleString("en-US");
const fixed = (digits: number) => (value: number) => value.toFixed(digits);

export function percentDelta(base: number, head: number): string {
  if (base === 0) return head === 0 ? "0.0%" : "n/a";
  const pct = ((head - base) / base) * 100;
  return `${pct > 0 ? "+" : pct < 0 ? "−" : ""}${Math.abs(pct).toFixed(1)}%`;
}

const METRICS: Array<[string, (lane: LaneSummary) => number, (value: number) => string]> = [
  ["CPU s (user+sys, process tree)", (lane) => lane.cpuS, fixed(2)],
  ["CPU s, fastest run", (lane) => lane.cpuMinS, fixed(2)],
  ["Max RSS MB", (lane) => lane.maxRssMb, fixed(0)],
  ["Instantiations", (lane) => lane.instantiations, integer],
  ["Types", (lane) => lane.types, integer],
  ["Check time s", (lane) => lane.checkS, fixed(3)],
  ["Wall s", (lane) => lane.wallS, fixed(2)],
  ["Generated bytes", (lane) => lane.generatedBytes, integer],
  ["Reply bytes (JSON)", (lane) => lane.replyBytes, integer],
  ["Mapped spans", (lane) => lane.spans, integer],
  ["Directive tuples", (lane) => lane.directives, integer],
  ["of which unmapped-gap ignores", (lane) => lane.ignoreGaps, integer],
  ["Diagnostics", (lane) => lane.diagnostics, integer],
];

const shortSha = (sha: string) => sha.slice(0, 12);
const diagnosticLine = ([file, kind, line, column, code, message]: Diagnostic) =>
  `${file}(${line},${column}): ${kind} ${code}: ${message.split("\n")[0]}`;
const LISTED = 25;

function identityLines(entry: Identity): string[] {
  const head = `- ${entry.corpus} / ${entry.host}: **${entry.status}**`;
  if (entry.status === "IDENTICAL") {
    return [
      `${head}, ${entry.base.count} diagnostics, sha256 \`${shortSha(entry.base.fingerprint)}\``,
    ];
  }
  const lines = [
    `${head}, base ${entry.base.count} → head ${entry.head.count} (+${entry.added.length} −${entry.removed.length})`,
  ];
  for (const [sign, list] of [
    ["+", entry.added],
    ["−", entry.removed],
  ] as const) {
    for (const diagnostic of list.slice(0, LISTED))
      lines.push(`  - ${sign} \`${diagnosticLine(diagnostic)}\``);
    if (list.length > LISTED)
      lines.push(`  - ${sign} … ${list.length - LISTED} more in results.json`);
  }
  return lines;
}

export function renderPairSummary(results: PairResults, summary: PairSummary): string {
  const packages = results.measurement.packages;
  const lines = [
    "## Type-check corpus pair",
    "",
    `base \`${shortSha(results.base.sha)}\` → head \`${shortSha(results.head.sha)}\`, median of ${results.runs} run(s) per lane, order alternated per run, on \`${results.measurement.runner}\`.`,
    `Hosts: \`tsc --noEmit --runExternalCode --extendedDiagnostics\` (typescript ${packages.typescript}) and \`tsrs --fast\` (tsrs ${packages.tsrs}); reference mapper @vue/content-mapper ${packages["@vue/content-mapper"]}; corpus packages vue ${packages.vue}, @types/node ${packages["@types/node"]}.`,
    "CPU is user+sys of the whole host process tree, mapper processes included (cutime+cstime from /proc/self/stat). Max RSS is GNU time %M: the peak of the largest single process in the tree. Generated bytes is the UTF-8 size of the virtual TS the mapper returned to the host, summed over every SFC. Reply bytes are the whole JSON result; the host decodes every span and directive tuple one by one, so tuple counts cost request wait beyond their bytes.",
    "",
  ];
  const corpora = [...new Map(results.lanes.map((lane) => [lane.corpus, lane])).values()];
  const hosts = [...new Set(results.lanes.map((lane) => lane.host))];
  for (const corpus of corpora) {
    const lane = (host: string, variant: LaneResult["variant"]) =>
      summary.lanes.find(
        (entry) =>
          entry.corpus === corpus.corpus && entry.host === host && entry.variant === variant,
      ) as LaneSummary;
    const header = [`${corpus.corpusLabel} (${corpus.sfcs} SFC)`];
    for (const host of hosts)
      header.push(`${host} Vize base`, `${host} Vize head`, "Δ", `${host} Vue`);
    lines.push(
      `| ${header.join(" | ")} |`,
      `|${" --- |".repeat(1)}${" ---: |".repeat(header.length - 1)}`,
    );
    for (const [label, pick, format] of METRICS) {
      const row = [label];
      for (const host of hosts) {
        const [base, head, vue] = [
          lane(host, "vize-base"),
          lane(host, "vize-head"),
          lane(host, "vue"),
        ];
        const identity = summary.identity.find(
          (e) => e.corpus === corpus.corpus && e.host === host,
        );
        const delta =
          label === "Diagnostics" && identity
            ? identity.status === "IDENTICAL"
              ? "IDENTICAL"
              : `+${identity.added.length} −${identity.removed.length}`
            : percentDelta(pick(base), pick(head));
        row.push(format(pick(base)), format(pick(head)), delta, format(pick(vue)));
      }
      lines.push(`| ${row.join(" | ")} |`);
    }
    lines.push("");
  }
  lines.push("### Diagnostic identity (Vize base vs head)", "");
  for (const entry of summary.identity) lines.push(...identityLines(entry));
  if (results.references.length > 0) {
    lines.push(
      "",
      `### Agreement with vue-tsc ${packages["vue-tsc"]} (typescript ${packages["vue-tsc typescript"]})`,
      "",
      "One untimed vue-tsc run per corpus on the tsconfig both mapper configs extend, so all three read the same `vueCompilerOptions`. Diagnostics match on (file, kind, line, column, code, message). Lists are in `diagnostics/<corpus>-vue-tsc.json`.",
      "",
      "| corpus | host | lane | lane total | shared | lane only | vue-tsc only |",
      "| --- | --- | --- | ---: | ---: | ---: | ---: |",
    );
    for (const entry of summary.agreement) {
      const total = entry.shared + entry.laneOnly;
      lines.push(
        `| ${entry.corpus} | ${entry.host} | ${entry.variant} | ${integer(total)} | ${integer(entry.shared)} | ${integer(entry.laneOnly)} | ${integer(entry.referenceOnly)} |`,
      );
    }
    lines.push("");
    for (const reference of results.references) {
      lines.push(
        `- ${reference.corpus} / ${reference.tool}: ${reference.diagnostics.length} diagnostics, CPU ${fixed(2)(reference.cpuMs / 1000)} s, max RSS ${fixed(0)(reference.maxRssKb / 1024)} MB, sha256 \`${shortSha(reference.fingerprint)}\``,
      );
    }
  }
  if (summary.unstable.length > 0) {
    lines.push("", `**Unstable diagnostics across runs:** ${summary.unstable.join(", ")}`);
  }
  return `${lines.join("\n")}\n`;
}
