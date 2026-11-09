/**
 * Ranked Markdown tables for a check-cost attribution, with base → head
 * deltas when a base attribution is given. Sampled ms come from one traced
 * run (1 ms resolution, noisy); types created are deterministic.
 */

import type { Attribution, KeyCost } from "./typecheck-cost-attribution.ts";

const int = (value: number) => Math.round(value).toLocaleString("en-US");
const signed = (value: number) => (Math.round(value) > 0 ? "+" : "") + int(value);
const pct = (value: number, of: number) => (of > 0 ? `${((100 * value) / of).toFixed(1)}%` : "n/a");
const cell = (text: string) => text.replace(/\|/gu, "\\|").replace(/\n/gu, " ");
const code = (text: string) => (text ? `\`${cell(text).replace(/`/gu, "'")}\`` : "");

const KIND_LABEL: Record<string, string> = {
  helper: "shared helper",
  "local-generic": "local generic alias",
  sfc: "SFC projection",
  vue: "vue types",
  lib: "TS lib",
  ts: "project .ts",
  pkg: "package",
};

function origin(key: string): { kind: string; name: string } {
  const [kind, name] = key.split("\t");
  if (name == null) return { kind: "anonymous", name: kind ?? key };
  return { kind: KIND_LABEL[kind ?? ""] ?? kind ?? "", name };
}

const EMPTY: KeyCost = { created: 0, createdFor: 0, inclusiveMs: 0, varianceMs: 0 };

function keyTable(
  title: string,
  head: Attribution,
  base: Attribution | undefined,
  keys: string[],
): string[] {
  const deltas = base != null;
  return [
    `#### ${title}`,
    "",
    `| origin | kind | types created for | ${deltas ? "Δ | " : ""}declared here | ms on type stack | ${deltas ? "Δ ms | " : ""}variance ms |`,
    `| --- | --- | ---: | ${deltas ? "---: | " : ""}---: | ---: | ${deltas ? "---: | " : ""}---: |`,
    ...keys.map((key) => {
      const now = head.keys[key] ?? EMPTY;
      const before = base?.keys[key] ?? EMPTY;
      const { kind, name } = origin(key);
      return `| ${code(name)} | ${kind} | ${int(now.createdFor)} | ${deltas ? `${signed(now.createdFor - before.createdFor)} | ` : ""}${int(now.created)} | ${int(now.inclusiveMs)} | ${deltas ? `${signed(now.inclusiveMs - before.inclusiveMs)} | ` : ""}${int(now.varianceMs)} |`;
    }),
    "",
  ];
}

export function renderAttribution(
  corpus: string,
  head: Attribution,
  base?: Attribution,
  top = 20,
): string {
  const deltas = base != null;
  const familyKeys = [
    ...new Set([...Object.keys(head.families), ...Object.keys(base?.families ?? {})]),
  ]
    .sort((a, b) => (head.families[b] ?? 0) - (head.families[a] ?? 0))
    .slice(0, top);
  const allKeys = [...new Set([...Object.keys(head.keys), ...Object.keys(base?.keys ?? {})])];
  const rank = (score: (cost: KeyCost) => number) =>
    allKeys
      .filter((key) => score(head.keys[key] ?? EMPTY) > 0)
      .sort((a, b) => score(head.keys[b] ?? EMPTY) - score(head.keys[a] ?? EMPTY))
      .slice(0, top);
  const generic = allKeys
    .filter((key) => key.startsWith("local-generic\t"))
    .sort((a, b) => (head.keys[b]?.createdFor ?? 0) - (head.keys[a]?.createdFor ?? 0))
    .slice(0, top);
  const siblings = Object.entries(head.siblings)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 10);
  const summaryDelta = (now: number, before: number | undefined) =>
    before == null ? "" : ` (base ${int(before)}, ${signed(now - before)})`;
  return [
    `### Check-cost attribution: ${corpus}`,
    "",
    `Traced tsc run, ${head.checkers} checkers: ${int(head.checkMs)} sampled check ms${summaryDelta(head.checkMs, base?.checkMs)}, ${int(head.sfcMs)} in SFC projections (${pct(head.sfcMs, head.checkMs)}); ${int(head.typesCreated)} types created${summaryDelta(head.typesCreated, base?.typesCreated)}. Ms are 1 ms samples of one traced run (tracing slows tsc, so read shares and large deltas); type counts are deterministic. A type is "created for" the first helper or projection origin up its creator chain (its creator being the first type created after it that references it); "declared here" counts its own declaration.`,
    "",
    "#### Projection families (statement being checked)",
    "",
    `| family | ms | share | ${deltas ? "base ms | Δ ms |" : ""}`,
    `| --- | ---: | ---: | ${deltas ? "---: | ---: |" : ""}`,
    ...familyKeys.map((family) => {
      const now = head.families[family] ?? 0;
      const before = base?.families[family] ?? 0;
      return `| ${cell(family)} | ${int(now)} | ${pct(now, head.checkMs)} | ${deltas ? `${int(before)} | ${signed(now - before)} |` : ""}`;
    }),
    "",
    ...keyTable(
      "Origins by check time (helpers, aliases, declarations)",
      head,
      base,
      rank((cost) => cost.inclusiveMs),
    ),
    ...keyTable(
      "Origins by types created for them",
      head,
      base,
      rank((cost) => cost.createdFor),
    ),
    ...keyTable(
      "Local generic aliases (per-file instantiations the program-wide cache cannot share)",
      head,
      base,
      generic,
    ),
    "#### Repeated per checker: variance measurement",
    "",
    "| alias | checkers | measurements | ms | example |",
    "| --- | ---: | ---: | ---: | --- |",
    ...head.variance.slice(0, 10).map((row) => {
      const { kind, name } = origin(row.key);
      return `| ${code(name)} (${kind}) | ${row.checkers} | ${row.runs} | ${int(row.ms)} | ${code(row.example.slice(0, 80))} |`;
    }),
    "",
    "#### Repeated per checker: relations between members of one union (subtype reduction)",
    "",
    "| union origin | ms |",
    "| --- | ---: |",
    ...siblings.map(
      ([key, ms]) => `| ${code(origin(key).name)} (${origin(key).kind}) | ${int(ms)} |`,
    ),
    "",
    "#### Anonymous types no later type references (built and dropped mid-check)",
    "",
    "| type | count |",
    "| --- | ---: |",
    ...head.unowned.slice(0, 10).map(([shape, count]) => `| ${code(shape)} | ${int(count)} |`),
    "",
    "#### Hottest relations (outermost structuredTypeRelatedTo)",
    "",
    "| source → target | checkers | ms |",
    "| --- | ---: | ---: |",
    ...head.relations.slice(0, 10).map(
      (row) =>
        `| ${code(
          row.key
            .split(" → ")
            .map((key) => origin(key).name)
            .join(" → "),
        )} | ${row.checkers} | ${int(row.ms)} |`,
    ),
    "",
    "#### Slowest SFCs (sampled ms, dominant family)",
    "",
    "| SFC | ms | family |",
    "| --- | ---: | --- |",
    ...head.files
      .slice(0, 10)
      .map((row) => `| ${code(row.file)} | ${int(row.ms)} | ${cell(row.family)} |`),
    "",
  ].join("\n");
}
