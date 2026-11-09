/**
 * Planted diagnostics for the real-corpus type-check pair. Each mutation is
 * one text edit in a real misskey or n8n component with the one diagnostic it
 * must produce. A mutated run must report every plant exactly once at its
 * position and code, and report every other diagnostic as the unmutated run
 * of the same build does. The sets live in tools/benchmarks/typecheck-corpora
 * and are pinned to the corpus revisions the lane checks out.
 */

import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { type Diagnostic, diffDiagnostics } from "./typecheck-corpus-pair-parse.ts";

const MUTATIONS_DIR = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "typecheck-corpora",
  "mutations",
);

/** Every construct family the template projection checks; each set covers all of them. */
export const MUTATION_FAMILIES = {
  "interpolation-identifier": "template interpolation identifier typo",
  "typed-member": "member access on a typed ref or store",
  "prop-static": "static prop with a wrong value type",
  "prop-bound": "bound prop with a wrong value type",
  "prop-missing": "missing required prop",
  "prop-unknown": "unknown prop on a strict component",
  "listener-handler": "@event handler with a wrong parameter type",
  "listener-inline": "inline @event handler with a wrong payload use",
  "emit-call": "defineEmits call with a wrong argument",
  "slot-prop-missing": "slot prop that does not exist",
  "slot-prop-type": "slot prop used with a wrong type",
  "v-model": "v-model type mismatch",
  "v-for-member": "v-for item member typo",
  "v-if-narrowing": "v-if narrowing misuse",
  "template-ref": "template ref member typo",
  fallthrough: "fallthrough attribute type mismatch",
  generic: "generic component type argument mismatch",
  "script-setup": "<script setup> type error",
  "with-defaults": "withDefaults default type mismatch",
  "define-model": "defineModel misuse",
  "template-literal": "error inside a multi-line template literal in script setup",
  "compile-error": "Vue compile error beside every other plant (#12522)",
} as const;
export type Family = keyof typeof MUTATION_FAMILIES;

export type Expected = { line: number; column: number; code: string; message?: string };
export type Mutation = {
  id: string;
  family: Family;
  /** Project-relative path, as diagnostics spell it. */
  file: string;
  /** Text that occurs exactly once in the pinned file. */
  anchor: string;
  /** Same line count as the anchor, so no other position moves. */
  replacement: string;
  /** Position in the mutated file, inside the replacement. */
  expect: Expected;
  /** A tracked Vize bug: the host reports this plant `count` times until `issue` lands. */
  known?: { issue: number; count: number; hosts?: string[] };
};
export type MutationSet = {
  corpus: string;
  submodule: string;
  revision: string;
  /** Families this corpus cannot plant, with the reason. */
  absent?: Partial<Record<Family, string>>;
  mutations: Mutation[];
};

type Position = { line: number; column: number };
export type Region = { file: string; start: Position; end: Position };

export function loadMutationSet(corpus: string, dir = MUTATIONS_DIR): MutationSet {
  const set = JSON.parse(readFileSync(join(dir, `${corpus}.json`), "utf8")) as MutationSet;
  if (set.corpus !== corpus) throw new Error(`${corpus}.json declares corpus ${set.corpus}`);
  if (!/^[0-9a-f]{40}$/u.test(set.revision)) throw new Error(`${corpus}: revision is not a SHA`);
  const ids = new Set<string>();
  for (const mutation of set.mutations) {
    if (ids.has(mutation.id)) throw new Error(`${corpus}: duplicate mutation id ${mutation.id}`);
    ids.add(mutation.id);
    if (!Object.hasOwn(MUTATION_FAMILIES, mutation.family)) {
      throw new Error(`${mutation.id}: unknown family ${mutation.family}`);
    }
    if (!/^(?:TS|vize)\d+$/u.test(mutation.expect.code)) {
      throw new Error(`${mutation.id}: bad code`);
    }
    const lines = (text: string) => text.split("\n").length;
    if (lines(mutation.anchor) !== lines(mutation.replacement)) {
      throw new Error(`${mutation.id}: the replacement changes the line count`);
    }
  }
  const missing = Object.keys(MUTATION_FAMILIES).filter(
    (family) =>
      !set.mutations.some((mutation) => mutation.family === family) &&
      !set.absent?.[family as Family],
  );
  if (missing.length > 0) throw new Error(`${corpus}: no mutation for ${missing.join(", ")}`);
  return set;
}

function positionAt(text: string, offset: number): Position {
  const before = text.slice(0, offset);
  const line = before.split("\n").length;
  return { line, column: offset - (before.lastIndexOf("\n") + 1) + 1 };
}

function offsetAt(text: string, { line, column }: Position): number {
  let offset = 0;
  for (let current = 1; current < line; current++) {
    offset = text.indexOf("\n", offset) + 1;
    if (offset === 0) return -1;
  }
  return offset + column - 1;
}

/**
 * Apply one file's mutations. Each anchor must occur exactly once, edits may
 * not share a line, and each expected position must fall inside its own
 * replacement in the mutated text.
 */
export function mutateFile(
  file: string,
  text: string,
  mutations: Mutation[],
): { text: string; regions: Map<string, Region> } {
  const edits = mutations
    .map((mutation) => {
      const start = text.indexOf(mutation.anchor);
      if (start < 0) throw new Error(`${mutation.id}: anchor not found in ${file}`);
      if (text.indexOf(mutation.anchor, start + 1) >= 0) {
        throw new Error(`${mutation.id}: anchor occurs more than once in ${file}`);
      }
      return { mutation, start, end: start + mutation.anchor.length };
    })
    .sort((a, b) => a.start - b.start);
  let out = "";
  let cursor = 0;
  const spans: Array<{ mutation: Mutation; start: number; end: number }> = [];
  for (const edit of edits) {
    if (edit.start < cursor) throw new Error(`${edit.mutation.id}: overlaps another mutation`);
    const previous = spans.at(-1);
    if (previous && positionAt(text, edit.start).line === positionAt(text, cursor).line) {
      throw new Error(`${edit.mutation.id}: shares a line with ${previous.mutation.id}`);
    }
    out += text.slice(cursor, edit.start);
    spans.push({
      mutation: edit.mutation,
      start: out.length,
      end: out.length + edit.mutation.replacement.length,
    });
    out += edit.mutation.replacement;
    cursor = edit.end;
  }
  out += text.slice(cursor);
  const regions = new Map<string, Region>();
  for (const span of spans) {
    const offset = offsetAt(out, span.mutation.expect);
    if (offset < span.start || offset >= span.end) {
      throw new Error(
        `${span.mutation.id}: expected ${span.mutation.expect.line}:${span.mutation.expect.column} is outside its replacement`,
      );
    }
    regions.set(span.mutation.id, {
      file,
      start: positionAt(out, span.start),
      end: positionAt(out, span.end),
    });
  }
  return { text: out, regions };
}

/** Mutated sources per file and each plant's replaced region. */
export function mutateSources(
  set: MutationSet,
  read: (file: string) => string,
): { files: Map<string, string>; regions: Map<string, Region> } {
  const byFile = Map.groupBy(set.mutations, (mutation) => mutation.file);
  const files = new Map<string, string>();
  const regions = new Map<string, Region>();
  for (const [file, mutations] of byFile) {
    const result = mutateFile(file, read(file), mutations);
    files.set(file, result.text);
    for (const [id, region] of result.regions) regions.set(id, region);
  }
  return { files, regions };
}

const inRegion = (diagnostic: Diagnostic, region: Region): boolean => {
  const line = Number(diagnostic[2]);
  const column = Number(diagnostic[3]);
  const after = (a: Position) => line > a.line || (line === a.line && column >= a.column);
  const before = (b: Position) => line < b.line || (line === b.line && column < b.column);
  return diagnostic[0] === region.file && after(region.start) && before(region.end);
};

export type PlantResult = {
  /** Diagnostics at the expected file, position and code (and message substring). */
  count: number;
  /** Other new diagnostics inside the replacement, as `line:column code`. */
  near: string[];
};
export type LaneJudgement = {
  plants: Record<string, PlantResult>;
  /** New and lost diagnostics outside the plants, against the unmutated run. */
  collateral: { added: Diagnostic[]; removed: Diagnostic[] };
};

export function judgeLane(
  set: MutationSet,
  regions: Map<string, Region>,
  mutated: Diagnostic[],
  pristine: Diagnostic[],
): LaneJudgement {
  const { added, removed } = diffDiagnostics(pristine, mutated);
  let rest = added;
  const plants: Record<string, PlantResult> = {};
  for (const mutation of set.mutations) {
    const { line, column, code, message } = mutation.expect;
    const hit = (d: Diagnostic) =>
      d[0] === mutation.file &&
      d[2] === String(line) &&
      d[3] === String(column) &&
      d[4] === code &&
      (message == null || d[5].includes(message));
    const region = regions.get(mutation.id);
    if (!region) throw new Error(`${mutation.id}: no region`);
    plants[mutation.id] = {
      count: rest.filter(hit).length,
      near: rest
        .filter((d) => !hit(d) && inRegion(d, region))
        .map((d) => `${d[2]}:${d[3]} ${d[4]}`),
    };
    rest = rest.filter((d) => !hit(d));
  }
  return { plants, collateral: { added: rest, removed } };
}

/** How a reference checker answered a plant: exact, another diagnostic in the replacement, or none. */
export function referenceAnswer(judgement: LaneJudgement, id: string): string {
  const plant = judgement.plants[id];
  if (!plant) throw new Error(`${id}: not judged`);
  if (plant.count > 0) return plant.count === 1 ? "exact" : `exact ${plant.count}x`;
  return plant.near.length > 0
    ? `near ${[...new Set(plant.near.map((n) => n.split(" ")[1]))].join(" ")}`
    : "-";
}

const knownFor = (mutation: Mutation, host: string) =>
  mutation.known && (!mutation.known.hosts || mutation.known.hosts.includes(host))
    ? mutation.known
    : undefined;

/**
 * The head gate, per host: every plant exactly once (or as its known issue
 * pins it), no plant base catches lost, and no collateral diagnostic.
 */
export function gateMutations(
  set: MutationSet,
  base: Record<string, LaneJudgement>,
  head: Record<string, LaneJudgement>,
): { failures: string[]; flips: string[] } {
  const failures: string[] = [];
  const flips: string[] = [];
  for (const [host, lane] of Object.entries(head)) {
    for (const mutation of set.mutations) {
      const count = lane.plants[mutation.id]?.count ?? 0;
      const baseCount = base[host]?.plants[mutation.id]?.count;
      const known = knownFor(mutation, host);
      const where = `${set.corpus} ${host} ${mutation.id}`;
      if (count === 1) {
        if (known) flips.push(`${where}: reported once; #${known.issue} landed, drop "known"`);
      } else if (baseCount === 1) {
        failures.push(`${where}: head reports ${count}x, base reports it once`);
      } else if (!known || known.count !== count) {
        failures.push(
          `${where}: head reports ${count}x, expected once${known ? ` or ${known.count}x (#${known.issue})` : ""}`,
        );
      }
    }
    const { added, removed } = lane.collateral;
    if (added.length > 0 || removed.length > 0) {
      const sample = [
        ...added.map((d) => `+${d.slice(0, 5).join(":")}`),
        ...removed.map((d) => `-${d.slice(0, 5).join(":")}`),
      ];
      failures.push(
        `${set.corpus} ${host}: ${added.length} collateral diagnostics added and ${removed.length} lost: ${sample.slice(0, 5).join(", ")}`,
      );
    }
  }
  return { failures, flips };
}

export type CorpusMutationReport = {
  set: MutationSet;
  vize: { base: Record<string, LaneJudgement>; head: Record<string, LaneJudgement> };
  references: Record<string, LaneJudgement>;
};

const vizeCell = (mutation: Mutation, host: string, lane: LaneJudgement | undefined): string => {
  const plant = lane?.plants[mutation.id];
  if (!plant) return "n/a";
  const known = knownFor(mutation, host);
  const note = known && plant.count === known.count ? ` (#${known.issue})` : "";
  if (plant.count === 1) return "1";
  return `**${plant.count}x**${note}`;
};

export function renderMutations(
  reports: CorpusMutationReport[],
  verdict: { failures: string[]; flips: string[] },
): string {
  const lines = ["### Planted diagnostics", ""];
  lines.push(
    "Each plant edits a real component; Vize must report its diagnostic exactly once at the listed position, and every other diagnostic must match the unmutated run. Vue mapper (tsc) and vue-tsc are references: `exact`, `near <code>` (another diagnostic inside the edit) or `-`.",
    "",
  );
  for (const { set, vize, references } of reports) {
    const hosts = Object.keys(vize.head);
    const refs = Object.keys(references);
    const header = [
      "Plant",
      "Family",
      "Expected",
      ...hosts.flatMap((host) => [`base ${host}`, `head ${host}`]),
      ...refs,
    ];
    lines.push(
      `#### ${set.corpus} @ ${set.revision.slice(0, 10)} (${set.mutations.length} plants)`,
      "",
    );
    lines.push(`| ${header.join(" | ")} |`, `|${" --- |".repeat(header.length)}`);
    for (const mutation of set.mutations) {
      const { line, column, code } = mutation.expect;
      const row = [
        `\`${mutation.id}\``,
        mutation.family,
        `${mutation.file.split("/").at(-1)}:${line}:${column} ${code}`,
        ...hosts.flatMap((host) => [
          vizeCell(mutation, host, vize.base[host]),
          vizeCell(mutation, host, vize.head[host]),
        ]),
        ...refs.map((ref) => referenceAnswer(references[ref] as LaneJudgement, mutation.id)),
      ];
      lines.push(`| ${row.join(" | ")} |`);
    }
    const caught = (lane: LaneJudgement | undefined) =>
      lane ? Object.values(lane.plants).filter((plant) => plant.count === 1).length : 0;
    const collateral = (lane: LaneJudgement | undefined) =>
      lane ? `+${lane.collateral.added.length}/-${lane.collateral.removed.length}` : "n/a";
    lines.push("", "| Lane | Exactly once | Collateral |", "| --- | ---: | ---: |");
    for (const role of ["base", "head"] as const) {
      for (const host of hosts) {
        const lane = vize[role][host];
        lines.push(
          `| vize ${role} ${host} | ${caught(lane)}/${set.mutations.length} | ${collateral(lane)} |`,
        );
      }
    }
    for (const ref of refs) {
      lines.push(
        `| ${ref} | ${caught(references[ref])}/${set.mutations.length} | ${collateral(references[ref])} |`,
      );
    }
    for (const [family, reason] of Object.entries(set.absent ?? {})) {
      lines.push("", `No ${family} plant: ${reason}`);
    }
    lines.push("");
  }
  lines.push(verdict.failures.length === 0 ? "Gate: pass." : "Gate: **fail**.", "");
  for (const failure of verdict.failures) lines.push(`- ${failure}`);
  for (const flip of verdict.flips) lines.push(`- flip: ${flip}`);
  return `${lines.join("\n").trimEnd()}\n`;
}
