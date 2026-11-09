/**
 * Check-cost attribution for one tsgo `--generateTrace` run of a corpus
 * through Vize's mapper: 1 ms samples of every checker (typecheck-cost-trace)
 * attributed to (a) the helper or local alias that declared the types being
 * related, (b) the projection family of the statement being checked, and
 * (c) work every checker repeats (variance measurement, relations between
 * siblings of one union, the shape union subtype reduction compares). Types
 * created per origin come from the types files (typecheck-cost-types).
 *
 *   node typecheck-cost-attribution.ts --trace <dir> --root <project> \
 *     [--generated <probe dump>] --out <attribution.json>
 *   node typecheck-cost-attribution.ts --pair <work> --out <dir>   (corpus pair layout)
 */

import { existsSync, readdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import {
  enclosingDeclaration,
  helperDeclarations,
  lineOf,
  localAlias,
  normalizeAlias,
  offsetOf,
  sectionStarts,
  siteFamily,
  skipTrivia,
  sourceText,
} from "./typecheck-cost-families.ts";
import {
  type Frame,
  type TraceEvent,
  checkerFrames,
  frameTypeIds,
  isTypeFrame,
  readLegend,
  statementChildren,
  tickStacks,
  varianceSpans,
} from "./typecheck-cost-trace.ts";
import { type LoadedText, familyLabel, siteResolver } from "./typecheck-cost-sites.ts";
import {
  type TypeDescriptor,
  type TypeRecord,
  censusTypes,
  pairKey,
} from "./typecheck-cost-types.ts";
import { renderAttribution } from "./typecheck-cost-report.ts";

/** Types created by declared origin and for a Vize-owned origin, and sampled ms. */
export type KeyCost = {
  created: number;
  createdFor: number;
  inclusiveMs: number;
  varianceMs: number;
};
export type DuplicateRow = {
  key: string;
  ms: number;
  runs: number;
  checkers: number;
  example: string;
};
export type Attribution = {
  schemaVersion: 1;
  checkers: number;
  /** Sampled ms inside checkSourceFile, summed over checkers. */
  checkMs: number;
  sfcMs: number;
  typesCreated: number;
  /** Ms by the family of the statement being checked (SFCs) or file kind. */
  families: Record<string, number>;
  /** Origin key → types created and ms with that origin anywhere on the type stack. */
  keys: Record<string, KeyCost>;
  variance: DuplicateRow[];
  relations: DuplicateRow[];
  /** Union origin key → ms relating two of its members. */
  siblings: Record<string, number>;
  /** Most frequent anonymous types without a creator (flags: display). */
  unowned: Array<[shape: string, count: number]>;
  files: Array<{ file: string; ms: number; family: string }>;
};

/**
 * Trace paths with their on-disk spelling: tsgo lowercases types.json paths on
 * case-insensitive hosts, where `existsSync` would accept them unchanged.
 */
function caseResolver(): (path: string) => string {
  const listings = new Map<string, Map<string, string>>();
  const cache = new Map<string, string>();
  const resolveCase = (path: string): string => {
    const parent = dirname(path);
    if (parent === path) return path;
    const hit = cache.get(path);
    if (hit != null) return hit;
    const dir = resolveCase(parent);
    let listing = listings.get(dir);
    if (!listing) {
      listing = new Map();
      const entries = existsSync(dir) ? readdirSync(dir) : [];
      for (const entry of entries) listing.set(entry.toLowerCase(), entry);
      // An exact spelling wins over a case-folded collision.
      for (const entry of entries) listing.set(entry, entry);
      listings.set(dir, listing);
    }
    const name = basename(path);
    const resolved = join(dir, listing.get(name) ?? listing.get(name.toLowerCase()) ?? name);
    cache.set(path, resolved);
    return resolved;
  };
  return resolveCase;
}

const isSfc = (path: string) => path.endsWith(".vue") || path.endsWith(".vue.tsx");
const isHelper = (path: string) => /[\\/]node_modules[\\/]vize[\\/]/u.test(path);
const OWNED = /^(helper|sfc|local-generic)\t/u;

type Lane = { traceDir: string; root: string; generated?: string };

export async function attributeLane(input: Lane): Promise<Attribution> {
  // Trace paths are real paths; compare against the real root.
  const lane = {
    ...input,
    root: realpathSync(input.root),
    ...(input.generated ? { generated: realpathSync(input.generated) } : {}),
  };
  const resolveCase = caseResolver();
  const texts = new Map<string, LoadedText | null>();
  const textOf = (tracePath: string) => {
    const path = resolveCase(tracePath);
    if (!texts.has(path)) {
      const rel = relative(lane.root, path);
      const generated =
        lane.generated && path.endsWith(".vue") && !rel.startsWith("..")
          ? join(lane.generated, `${rel}.tsx`)
          : path;
      const file = existsSync(generated) ? generated : null;
      const text = file ? sourceText(readFileSync(file)) : null;
      texts.set(path, text ? { text, sections: sectionStarts(text) } : null);
    }
    return texts.get(path) ?? null;
  };
  const helpers = new Map<string, Array<[number, string]>>();
  const memo = new Map<string, string | null>();
  const origin = (record: TypeRecord): string | null => {
    const declaration = record.firstDeclaration;
    if (!declaration) return null;
    const { path, start } = declaration;
    const symbol = record.symbolName ?? "?";
    const memoKey = `${path}:${start.line}:${start.character}:${symbol}`;
    const cached = memo.get(memoKey);
    if (cached !== undefined) return cached;
    let key: string;
    if (isHelper(path)) {
      const loaded = textOf(path);
      if (loaded && !helpers.has(path)) helpers.set(path, helperDeclarations(loaded.text));
      // The node's full start includes the previous line's trivia; classify its own line.
      const offset = loaded
        ? skipTrivia(loaded.text, offsetOf(loaded.text, start.line, start.character))
        : 0;
      const own = loaded ? lineOf(loaded.text, offset) : 0;
      key = `helper\t${enclosingDeclaration(helpers.get(path) ?? [], own) ?? basename(path)}`;
    } else if (isSfc(path)) {
      const loaded = textOf(path);
      if (!loaded) key = `sfc\t(no text)`;
      else {
        const offset = offsetOf(loaded.text, start.line, start.character);
        const site = siteFamily(loaded.text, loaded.sections, offset);
        const alias = localAlias(loaded.text, offset);
        // The component's own type alias is named after the SFC.
        const name =
          alias && site.section === "component-export" && !alias.name.startsWith("$")
            ? "<SFC>Component"
            : alias && normalizeAlias(alias.name);
        key = alias
          ? `${alias.generic ? "local-generic" : "sfc"}\t${familyLabel(site)} ${name}`
          : `sfc\t${familyLabel(site)}`;
      }
    } else if (/[\\/]lib\.[\w.]*d\.ts$/u.test(path)) key = `lib\t${symbol}`;
    else if (/[\\/]node_modules[\\/](@vue[\\/]|vue[\\/])/u.test(path)) key = `vue\t${symbol}`;
    else if (/[\\/]node_modules[\\/]/u.test(path)) {
      key = `pkg\t${(/[\\/]node_modules[\\/]((?:@[^\\/]+[\\/])?[^\\/]+)/u.exec(path)?.[1] ?? "?").replace(/\\/gu, "/")}`;
    } else key = `ts\t${symbol}`;
    memo.set(memoKey, key);
    return key;
  };

  const legend = readLegend(lane.traceDir);
  const events = JSON.parse(
    readFileSync((legend[0] ?? { tracePath: join(lane.traceDir, "trace.json") }).tracePath, "utf8"),
  ) as TraceEvent[];
  const frames = checkerFrames(events);
  const result: Attribution = {
    schemaVersion: 1,
    checkers: frames.size,
    checkMs: 0,
    sfcMs: 0,
    typesCreated: 0,
    families: {},
    keys: {},
    variance: [],
    relations: [],
    siblings: {},
    unowned: [],
    files: [],
  };
  const unowned = new Map<string, number>();
  const keyCost = (key: string) =>
    (result.keys[key] ??= { created: 0, createdFor: 0, inclusiveMs: 0, varianceMs: 0 });
  const bump = (record: Record<string, number>, key: string, by = 1) => {
    record[key] = (record[key] ?? 0) + by;
  };
  const variance = new Map<string, DuplicateRow & { seen: Set<number> }>();
  const relations = new Map<string, DuplicateRow & { seen: Set<number> }>();
  const duplicate = (
    rows: Map<string, DuplicateRow & { seen: Set<number> }>,
    key: string,
    checker: number,
    ms: number,
    example: string,
    run: boolean,
  ) => {
    const row = rows.get(key) ?? { key, ms: 0, runs: 0, checkers: 0, example, seen: new Set() };
    row.ms += ms;
    if (run) row.runs++;
    row.seen.add(checker);
    row.checkers = row.seen.size;
    rows.set(key, row);
  };
  const files = new Map<string, Record<string, number>>();

  for (const entry of legend) {
    const checker = entry.checkerId;
    const own = frames.get(checker) ?? [];
    const stacks = tickStacks(own);
    const spans = varianceSpans(own);
    const siteOf = siteResolver(textOf, statementChildren(own));
    const wanted = new Set<number>();
    const pairs: Array<[number, number]> = [];
    const seenPairs = new Set<string>();
    for (const frame of own) {
      if (!isTypeFrame(frame)) continue;
      for (const id of frameTypeIds(frame)) wanted.add(id);
      const { sourceId, targetId } = frame.args;
      if (sourceId != null && targetId != null && !seenPairs.has(pairKey(sourceId, targetId))) {
        seenPairs.add(pairKey(sourceId, targetId));
        pairs.push([sourceId, targetId]);
      }
    }
    const census = existsSync(entry.typesPath)
      ? await censusTypes(entry.typesPath, origin, (key) => OWNED.test(key), wanted, pairs)
      : {
          created: new Map<string, number>(),
          createdFor: new Map<string, number>(),
          unowned: new Map<string, number>(),
          described: new Map<number, TypeDescriptor>(),
          siblings: new Map<string, string>(),
          total: 0,
        };
    result.typesCreated += census.total;
    for (const [key, count] of census.created) keyCost(key).created += count;
    for (const [key, count] of census.createdFor) keyCost(key).createdFor += count;
    for (const [shape, count] of census.unowned)
      unowned.set(shape, (unowned.get(shape) ?? 0) + count);
    // A lib or vue type is named by the Vize-owned origin it was created for.
    const keyOf = (id: number | undefined) => {
      const described = id == null ? undefined : census.described.get(id);
      if (!described) return "?";
      return OWNED.test(described.key) ? described.key : described.owner;
    };
    const display = (id: number | undefined) =>
      id == null ? "" : (census.described.get(id)?.display ?? "");

    for (const span of spans) {
      const ms = (span.end - span.start) / 1000;
      duplicate(variance, keyOf(span.args.id), checker, ms, display(span.args.id), true);
      keyCost(keyOf(span.args.id)).varianceMs += ms;
    }
    for (const [tick, stack] of stacks) {
      const file = stack[0] as Frame;
      const path = file.args.path ?? "?";
      result.checkMs++;
      const shares: Array<[string, number]> = !isSfc(path)
        ? [[isHelper(path) ? "helper files" : "non-SFC files", 1]]
        : siteOf(stack, tick);
      for (const [family, share] of shares) bump(result.families, family, share);
      if (isSfc(path)) {
        result.sfcMs++;
        const rel = relative(lane.root, resolveCase(path)).split(sep).join("/");
        const perFile = files.get(rel) ?? {};
        for (const [family, share] of shares) bump(perFile, family, share);
        files.set(rel, perFile);
      }
      const keys = new Set<string>();
      for (const frame of stack) {
        if (!isTypeFrame(frame)) continue;
        for (const id of frameTypeIds(frame)) {
          const described = census.described.get(id);
          if (described) keys.add(described.key).add(described.owner);
        }
      }
      for (const key of keys) keyCost(key).inclusiveMs++;
      const relation = stack.find((frame) => frame.name === "structuredTypeRelatedTo");
      if (relation) {
        const { sourceId, targetId } = relation.args;
        const key = `${keyOf(sourceId)} → ${keyOf(targetId)}`;
        duplicate(relations, key, checker, 1, `${display(sourceId)} → ${display(targetId)}`, false);
      }
      const sibling = stack
        .filter((frame) => frame.name === "structuredTypeRelatedTo")
        .map((frame) =>
          census.siblings.get(pairKey(frame.args.sourceId ?? -1, frame.args.targetId ?? -1)),
        )
        .find((union) => union != null);
      if (sibling) bump(result.siblings, sibling);
    }
  }
  const rows = (map: Map<string, DuplicateRow & { seen: Set<number> }>) =>
    [...map.values()].map(({ seen: _seen, ...row }) => row).sort((a, b) => b.ms - a.ms);
  result.variance = rows(variance).slice(0, 200);
  result.relations = rows(relations).slice(0, 200);
  result.unowned = [...unowned].sort((a, b) => b[1] - a[1]).slice(0, 50);
  result.files = [...files]
    .map(([file, byFamily]) => {
      const ranked = Object.entries(byFamily).sort((a, b) => b[1] - a[1]);
      return {
        file,
        ms: ranked.reduce((sum, [, ms]) => sum + ms, 0),
        family: ranked[0]?.[0] ?? "",
      };
    })
    .sort((a, b) => b.ms - a.ms)
    .slice(0, 100);
  return result;
}

const scriptPath = fileURLToPath(import.meta.url);

/** The corpus pair's traced probe lanes: `<work>/traces/<corpus>-tsc-vize-<role>`. */
export function pairLanes(
  work: string,
  out: string,
): Array<{ corpus: string; role: string } & Lane> {
  const traces = join(work, "traces");
  if (!existsSync(traces)) return [];
  return readdirSync(traces)
    .map((name) => /^(.+)-tsc-vize-(base|head)$/u.exec(name))
    .filter((match): match is RegExpExecArray => match != null)
    .map(([name, corpus, role]) => ({
      corpus: corpus as string,
      role: role as string,
      traceDir: join(traces, name as string),
      root: join(work, `${corpus}-${role}`),
      generated: join(out, "generated", name as string),
    }));
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      trace: { type: "string" },
      root: { type: "string" },
      generated: { type: "string" },
      pair: { type: "string" },
      out: { type: "string" },
    },
  });
  if (!values.out) throw new Error("--out is required");
  if (values.pair) {
    const out = resolve(values.out);
    const lanes = pairLanes(resolve(values.pair), out);
    if (lanes.length === 0) throw new Error(`no traced Vize lanes under ${values.pair}/traces`);
    const reports: Record<string, Partial<Record<string, Attribution>>> = {};
    for (const lane of lanes) {
      console.log(`attributing ${lane.corpus} ${lane.role}`);
      (reports[lane.corpus] ??= {})[lane.role] = await attributeLane(lane);
    }
    writeFileSync(join(out, "attribution.json"), `${JSON.stringify(reports)}\n`);
    writeFileSync(
      join(out, "attribution.md"),
      Object.entries(reports)
        .map(([corpus, { base, head }]) =>
          renderAttribution(corpus, head ?? base!, head ? base : undefined),
        )
        .join("\n"),
    );
    return;
  }
  if (!values.trace || !values.root) throw new Error("--trace and --root (or --pair) are required");
  const report = await attributeLane({
    traceDir: resolve(values.trace),
    root: resolve(values.root),
    ...(values.generated ? { generated: resolve(values.generated) } : {}),
  });
  writeFileSync(resolve(values.out), `${JSON.stringify(report)}\n`);
  console.log(renderAttribution(basename(values.root), report));
}

if (process.argv[1] && resolve(process.argv[1]) === scriptPath) await main();
