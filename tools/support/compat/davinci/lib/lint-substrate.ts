// The lint substrate ledger: each Patina rule, the level it belongs on
// (L1 syntax, L2 semantic, L1 script, L1 css), the engine that runs it on the
// default SFC path today, and the legacy crates its own code still reads.
// Everything is derived from source through the rule-parity matrix, so a
// migration PR shows up as row changes and nothing here is hand-maintained.

import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

import { walkRustFiles } from "./crates.ts";
import { formatTable } from "./markdown.ts";
import { byKey } from "./ordering.ts";
import { buildMatrix } from "./rule-parity-matrix.ts";
import type { Matrix } from "./rule-parity-matrix.ts";
import { RULES_DIR, repoRoot } from "./rule-parity-paths.ts";
import { stripRustComments } from "./rule-parity-rust-text.ts";
import { stripRust } from "./rust-source.ts";

export const LEDGER_REL = "docs/davinci/plan/lint-substrate-ledger.tsv";
export const FACADE_REL = "crates/vize_patina/src/linter/engine/sfc/facade.rs";
export const REGEN = "node tools/support/compat/davinci/lint-substrate.ts --write";

export const SUBSTRATES = ["L1 syntax", "L2 semantic", "L1 script", "L1 css"] as const;
export type Substrate = (typeof SUBSTRATES)[number];
export const STATUSES = ["native", "legacy-engine", "legacy-rule"] as const;
export type Status = (typeof STATUSES)[number];

/**
 * Engines a rule runs on once it is registered (configured rules register from
 * their options, so registration does not change the engine), and whether each one already reads the shared
 * levels. `markup-l2` is the markup facade over the L2 template artifact that
 * the one L1 template parse lowers to (`MarkupDocument::from_l2`); it serves
 * the L1 syntax substrate without a second parse. Everything else is fed by a
 * legacy producer: the Relief compatibility root, the Atelier SFC descriptor
 * and its script parses, lightningcss, the Corsa checker or the Musea parser.
 */
export const ENGINES = new Map<string, string>([
  ["markup-l2", "markup facade over the L2 template artifact (`MarkupDocument::from_l2`)"],
  ["markup-relief", "markup facade over the Relief compatibility root"],
  ["relief-visitor", "`Rule` template hooks over the Relief compatibility root"],
  ["sfc-descriptor", "`run_on_sfc` over the Atelier SFC descriptor"],
  ["script-oxc", "script rules over the Atelier SFC script parses"],
  ["style-l1", "`StyleRule` over the L1 style structure"],
  ["css-lightningcss", "`CssRule` over a lightningcss parse"],
  ["corsa", "type-aware rules served by the Corsa checker session"],
  ["musea-blocks", "Musea Art-file blocks"],
]);

/** The engines each substrate is finished on. */
const TARGET_ENGINES = new Map<Substrate, ReadonlySet<string>>([
  ["L1 syntax", new Set(["markup-l2"])],
  ["L2 semantic", new Set(["markup-l2"])],
  ["L1 script", new Set(["script-l1"])],
  ["L1 css", new Set(["style-l1"])],
]);

const LEGACY_CRATE_RE =
  /\bvize_(relief|croquis_cf|croquis|armature|atelier_core|atelier_sfc|atelier_jsx|atelier_dom|atelier_ssr|atelier_vapor)\b/g;

export interface LedgerRow {
  rule: string;
  file: string;
  substrate: Substrate;
  engine: string[];
  legacy: string[];
  status: Status;
}

/** `RULES` in the SFC facade: the rules the default SFC path runs on L2. */
export function facadeRules(): Set<string> {
  const source = stripRustComments(readFileSync(path.join(repoRoot, FACADE_REL), "utf8"));
  const anchor = /const\s+RULES\s*:\s*&\[&str\]\s*=\s*&\[([^\]]*)\]/u.exec(source);
  if (!anchor) throw new Error(`${FACADE_REL}: RULES list not found; the ledger model is stale`);
  const names = [...anchor[1].matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
  if (names.length === 0) throw new Error(`${FACADE_REL}: RULES list is empty`);
  return new Set(names);
}

/** Legacy crates named in a rule file or in its private submodule directory. */
function legacyCrates(ruleRel: string): string[] {
  const abs = path.join(RULES_DIR, ruleRel);
  const files = [abs];
  const dir = abs.replace(/\.rs$/u, "");
  if (existsSync(dir)) {
    files.push(...walkRustFiles(dir).filter((file) => !/_tests?\.rs$|\/tests?\//u.test(file)));
  }
  const crates = new Set<string>();
  for (const file of files) {
    const code = stripRust(readFileSync(file, "utf8"));
    for (const match of code.matchAll(LEGACY_CRATE_RE)) crates.add(match[1]);
  }
  return [...crates].sort(byKey);
}

function cssEngine(ruleRel: string): string {
  const code = stripRust(readFileSync(path.join(RULES_DIR, ruleRel), "utf8"));
  const style = /\bimpl\b[^{;]*\bStyleRule\s+for\b/u.test(code);
  const css = /\bimpl\b[^{;]*\bCssRule\s+for\b/u.test(code);
  if (style === css) {
    throw new Error(`${ruleRel}: a css rule must implement exactly one of StyleRule/CssRule`);
  }
  return style ? "style-l1" : "css-lightningcss";
}

export function buildLedger(matrix: Matrix = buildMatrix()): LedgerRow[] {
  const facade = facadeRules();
  const rows: LedgerRow[] = [];
  for (const rule of matrix.rules.values()) {
    const surfaces = new Set(rule.surfaces);
    const legacy = legacyCrates(rule.file);
    const semanticReads =
      legacy.includes("croquis") ||
      rule.croquis.size > 0 ||
      rule.ctxSites > 0 ||
      matrix.model.semanticTemplateRules.includes(rule.name);
    const engine: string[] = [];
    let substrate: Substrate;
    switch (rule.family) {
      case "template-family": {
        // Metadata-only `type/*` rules have no hooks: the checker session
        // reports them through the typescript-eslint name table.
        const corsa =
          surfaces.has("type-aware-corsa") ||
          (surfaces.has("none") && rule.name.startsWith("type/"));
        substrate = corsa || semanticReads ? "L2 semantic" : "L1 syntax";
        if (facade.has(rule.name)) engine.push("markup-l2");
        else if (surfaces.has("markup-facade")) engine.push("markup-relief");
        else if (surfaces.has("template-ast")) engine.push("relief-visitor");
        if (surfaces.has("sfc-source")) engine.push("sfc-descriptor");
        if (surfaces.has("script-oxc") || surfaces.has("script-source")) engine.push("script-oxc");
        if (corsa) engine.push("corsa");
        break;
      }
      case "script":
        substrate = semanticReads ? "L2 semantic" : "L1 script";
        engine.push("script-oxc");
        break;
      case "css":
        substrate = "L1 css";
        engine.push(cssEngine(rule.file));
        break;
      case "musea":
        substrate = "L1 syntax";
        engine.push("musea-blocks");
        break;
      default:
        throw new Error(`${rule.file}: unknown rule family ${String(rule.family)}`);
    }
    if (engine.length === 0) throw new Error(`${rule.file}: ${rule.name} has no engine`);
    for (const name of engine) {
      if (!ENGINES.has(name)) throw new Error(`${rule.file}: unknown engine ${name}`);
    }
    const target = TARGET_ENGINES.get(substrate)!;
    const status: Status =
      legacy.length > 0
        ? "legacy-rule"
        : engine.every((name) => target.has(name))
          ? "native"
          : "legacy-engine";
    rows.push({ rule: rule.name, file: rule.file, substrate, engine, legacy, status });
  }
  return rows.sort((left, right) => byKey(left.rule, right.rule));
}

const HEADER = ["rule", "substrate", "engine", "status", "legacy", "file"] as const;

/** The committed per-rule ledger; no totals, so independent rows never conflict. */
export function renderLedger(rows: readonly LedgerRow[]): string {
  const lines = [
    `# Generated by ${REGEN}; verify with --check. Rule paths are under crates/vize_patina/src/rules/.`,
    HEADER.join("\t"),
  ];
  for (const row of rows) {
    lines.push(
      [
        row.rule,
        row.substrate,
        row.engine.join("+"),
        row.status,
        row.legacy.length > 0 ? row.legacy.join(",") : "-",
        row.file,
      ].join("\t"),
    );
  }
  return `${lines.join("\n")}\n`;
}

function countBy<K extends string>(rows: readonly LedgerRow[], key: (row: LedgerRow) => K[]) {
  const counts = new Map<K, number>();
  for (const row of rows) for (const k of key(row)) counts.set(k, (counts.get(k) ?? 0) + 1);
  return counts;
}

/** The run-artifact summary: every count lives here, never in the ledger. */
export function renderSummary(rows: readonly LedgerRow[]): string {
  const out: string[] = [
    "<!-- GENERATED FILE — do not edit by hand.",
    "     Regenerate: node tools/support/compat/davinci/lint-substrate.ts --summary",
    `     Per-rule rows: ${LEDGER_REL} -->`,
    "",
    "# Lint substrate summary",
    "",
    "`products.linter` (#6848) is done when every rule runs on its level: syntax rules on L1, semantic rules on L2, script rules on the L1 program and style rules on the L1 style structure, with no legacy crate in the rule or its engine.",
    "",
    "## Rules by substrate and status",
    "",
  ];
  const statusRows: (string | number)[][] = SUBSTRATES.map((substrate) => {
    const of = rows.filter((row) => row.substrate === substrate);
    return [
      substrate,
      ...STATUSES.map((status) => of.filter((row) => row.status === status).length),
      of.length,
    ];
  });
  statusRows.push([
    "total",
    ...STATUSES.map((status) => rows.filter((row) => row.status === status).length),
    rows.length,
  ]);
  out.push(
    formatTable(
      ["substrate", ...STATUSES, "rules"],
      ["left", ...STATUSES.map(() => "right" as const), "right"],
      statusRows,
    ),
  );
  out.push("## Rules by engine", "");
  const engines = countBy(rows, (row) => row.engine);
  out.push(
    formatTable(
      ["engine", "rules", "reads"],
      ["left", "right", "left"],
      [...ENGINES].map(([name, reads]) => [name, engines.get(name) ?? 0, reads]),
    ),
  );
  out.push("## Legacy crates read by rule code", "");
  const legacy = countBy(rows, (row) => row.legacy);
  out.push(
    formatTable(
      ["crate", "rules"],
      ["left", "right"],
      [...legacy].sort((a, b) => byKey(a[0], b[0])).map(([name, count]) => [`vize_${name}`, count]),
    ),
  );
  return `${out.join("\n").replace(/\n+$/u, "")}\n`;
}

export function ledgerPath(): string {
  return path.join(repoRoot, LEDGER_REL);
}
