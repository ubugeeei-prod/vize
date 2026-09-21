// The Spolvero feed as a stage ladder: rungs (S1 Surface, S2 Disegno,
// S3 Impeto), their pages, a few counts read off the canonical pages, and the
// pass timeline in feed order. Pure data shaping over the negotiated feed -
// every fact shown comes from a page the compiler printed.

import type { SpolveroFeed, SpolveroPage } from "../../wasm/types/spolvero";
import type { PageKind } from "./folioLines";

export type RungId = "s1" | "s2" | "s3";

export interface LadderPage {
  /** Stable key: `stage/pass`, unique within one template's pages. */
  key: string;
  kind: PageKind;
  stage: string;
  pass: string;
  text: string;
  /** Short tab label within the rung. */
  label: string;
}

export interface Rung {
  id: RungId;
  /** Stage number as shown on the rail. */
  ordinal: string;
  name: string;
  pages: LadderPage[];
  /** Short facts read off the rung's pages, e.g. `8 ops`. */
  facts: string[];
}

export interface TimelineStep {
  key: string;
  rung: RungId;
  pass: string;
  /** Whether this step's page differs from the previous page of its rung. */
  changed: boolean;
  /** Whether the step is a lowering/parse (a new artifact), not a pass. */
  producer: boolean;
  /** Measured wall time in nanoseconds, when the run was profiled. */
  nanos: number | null;
}

export interface StageLadder {
  rungs: Rung[];
  timeline: TimelineStep[];
  /** The S1 page text: the authored template every span indexes. */
  template: string;
  /** Stage names the view does not know how to place, kept visible. */
  unplaced: string[];
}

const PAGE_KINDS: Record<string, { rung: RungId; kind: PageKind; label: string }> = {
  s1: { rung: "s1", kind: "surface", label: "Surface" },
  s2: { rung: "s2", kind: "disegno", label: "" },
  "s2-provenance": { rung: "s2", kind: "provenance", label: "Provenance" },
  s3: { rung: "s3", kind: "impeto", label: "Graph" },
  "s3-partition": { rung: "s3", kind: "partition", label: "Partition" },
  "s3-values": { rung: "s3", kind: "values", label: "Values" },
};

const RUNG_NAMES: Record<RungId, { ordinal: string; name: string }> = {
  s1: { ordinal: "S1", name: "Surface" },
  s2: { ordinal: "S2", name: "Disegno" },
  s3: { ordinal: "S3", name: "Impeto" },
};

function plural(count: number, noun: string): string {
  if (count === 1) return `1 ${noun}`;
  return `${count} ${noun}${noun.endsWith("s") ? "es" : "s"}`;
}

function sectionLines(text: string, section: string): string[] {
  const start = text.indexOf(`[${section}]\n`);
  if (start === -1) return [];
  const body = text.slice(start + section.length + 3);
  const end = body.indexOf("\n\n");
  return (end === -1 ? body : body.slice(0, end)).split("\n").filter(Boolean);
}

function rungFacts(id: RungId, pages: LadderPage[]): string[] {
  const byKind = (kind: PageKind) => pages.find((page) => page.kind === kind);
  switch (id) {
    case "s1": {
      const text = byKind("surface")?.text ?? "";
      const lines = text.endsWith("\n") ? text.split("\n").length - 1 : text.split("\n").length;
      return [plural(lines, "line")];
    }
    case "s2": {
      const trees = pages.filter((page) => page.kind === "disegno");
      const ops = /^ops=(\d+)$/m.exec(trees[0]?.text ?? "");
      const passes = Math.max(trees.length - 1, 0);
      return [plural(Number(ops?.[1] ?? 0), "op"), plural(passes, "pass")];
    }
    case "s3": {
      const graph = byKind("impeto")?.text ?? "";
      const partition = byKind("partition")?.text ?? "";
      const kinds = sectionLines(partition, "s3-partition-folio.ops");
      const dynamic = kinds.filter((line) => /\bkind=dynamic\b/.test(line)).length;
      return [plural(sectionLines(graph, "s3-folio.ops").length, "op"), `${dynamic} dynamic`];
    }
  }
}

function pageLabel(stage: string, pass: string): string {
  const known = PAGE_KINDS[stage];
  if (stage === "s2") return pass === "lower" ? "Lowered" : pass;
  return known?.label || `${stage}/${pass}`;
}

/**
 * Shape a negotiated feed's pages (for one file) into the ladder; `timings`
 * (from the profile export, keyed `stage/pass`) fills each step's wall time.
 */
export function buildLadder(
  feed: SpolveroFeed,
  path?: string,
  timings: ReadonlyMap<string, number> = new Map(),
): StageLadder {
  const pages: SpolveroPage[] = feed.pages.filter(
    (page) => path === undefined || page.path === path,
  );
  const grouped: Record<RungId, LadderPage[]> = { s1: [], s2: [], s3: [] };
  const unplaced: string[] = [];
  for (const page of pages) {
    const placement = PAGE_KINDS[page.stage];
    if (!placement) {
      if (!unplaced.includes(page.stage)) unplaced.push(page.stage);
      continue;
    }
    grouped[placement.rung].push({
      key: `${page.stage}/${page.pass}`,
      kind: placement.kind,
      stage: page.stage,
      pass: page.pass,
      text: page.text,
      label: pageLabel(page.stage, page.pass),
    });
  }

  const rungs = (Object.keys(grouped) as RungId[]).map((id) => ({
    id,
    ...RUNG_NAMES[id],
    pages: grouped[id],
    facts: grouped[id].length > 0 ? rungFacts(id, grouped[id]) : [],
  }));

  const timeline: TimelineStep[] = [];
  const previous: Partial<Record<RungId, string>> = {};
  for (const rung of rungs) {
    for (const page of rung.pages) {
      // The S3 partition and value pages come from the same lowering step as
      // the graph, and the provenance page records decisions across S2's
      // steps; the timeline shows steps, not pages.
      if (page.kind === "partition" || page.kind === "values" || page.kind === "provenance") {
        continue;
      }
      const producer = page.pass === "lower" || page.pass === "parse";
      timeline.push({
        key: page.key,
        rung: rung.id,
        pass: page.pass,
        changed: producer || previous[rung.id] !== page.text,
        producer,
        nanos: timings.get(page.key) ?? null,
      });
      previous[rung.id] = page.text;
    }
  }

  return {
    rungs,
    timeline,
    template: grouped.s1[0]?.text ?? "",
    unplaced,
  };
}
