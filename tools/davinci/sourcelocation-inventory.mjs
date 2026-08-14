#!/usr/bin/env node
// SourceLocation consumer inventory (Davinci P0-9).
//
// Counts every textual read of the relief `SourceLocation` members that the
// Davinci `Span` type (crates/vize_carton/src/span.rs) deletes —
// `source`, `start.line`, `start.column`, `end.line`, `end.column` — across
// `crates/*/src/**/*.rs`, grouped by crate and member, and emits the
// migration map P1 executes as davinci-road/plan/sourcelocation-inventory.md.
//
// Resolution is textual, not type-aware: comments and string literals are
// stripped first (lib/rust-source.mjs), then member paths are counted on
// loc-shaped receivers only — chained `.loc.<member>` / `.loc().<member>` /
// `.location.<member>` field or method accesses, and bare locals whose
// identifier is `loc`, `location`, or ends in `_loc` / `_location`. The
// artifact documents the known imprecision. Citations in the migration-group
// prose are looked up at generation time (file + anchor regex), so a moved
// or deleted consumer fails regeneration instead of going silently stale.
//
// Usage:
//   node tools/davinci/sourcelocation-inventory.mjs --write   # regenerate
//   node tools/davinci/sourcelocation-inventory.mjs --check   # diff committed
//
// Node builtins only. Output is deterministic (stable sort everywhere,
// no timestamps, no absolute paths).

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import { discoverCrates, walkRustFiles } from "./lib/crates.mjs";
import { formatTable } from "./lib/markdown.mjs";
import { repoRoot } from "./lib/paths.mjs";
import { stripRust } from "./lib/rust-source.mjs";

const ARTIFACT_REL = "davinci-road/plan/sourcelocation-inventory.md";
const ARTIFACT = path.join(repoRoot, ARTIFACT_REL);
const REGEN_COMMAND = "node tools/davinci/sourcelocation-inventory.mjs --write";

/** The five member paths `Span` deletes, in table-column order. */
const MEMBERS = ["source", "start.line", "start.column", "end.line", "end.column"];

/** Offset member paths that survive the migration as `span.start`/`span.end`. */
const OFFSET_MEMBERS = ["start.offset", "end.offset"];

/** Is `ident` a loc-shaped receiver or field name? */
function isLocShaped(ident) {
  return (
    ident === "loc" || ident === "location" || ident.endsWith("_loc") || ident.endsWith("_location")
  );
}

/**
 * Scan stripped Rust text for reads of the given member paths through
 * loc-shaped receivers. Returns [{ index, member }].
 *
 * One regex pass finds `<ident>[()].<member>` shapes, capturing whether the
 * receiver is itself preceded by `.` (a field/method in a chain, e.g.
 * `node.loc.source`) or stands bare (a local, e.g. `loc.source`); the
 * receiver identifier is then filtered to loc-shaped names. Bare receivers
 * additionally require a non-identifier, non-`.` character before them so a
 * rejected chain segment is never recounted as a bare local.
 */
function scanMembers(stripped, members) {
  const memberAlt = members.map((m) => m.replaceAll(".", "\\.")).join("|");
  const re = new RegExp(
    `(\\.)?([A-Za-z_][A-Za-z0-9_]*)(\\(\\))?\\.(${memberAlt})(?![A-Za-z0-9_])`,
    "g",
  );
  const sites = [];
  let m;
  while ((m = re.exec(stripped)) !== null) {
    const [, chained, receiver, , member] = m;
    if (!isLocShaped(receiver)) continue;
    if (!chained) {
      const before = m.index === 0 ? "" : stripped[m.index - 1];
      if (/[A-Za-z0-9_.]/.test(before)) continue;
    }
    sites.push({ index: m.index, member });
  }
  return sites;
}

function lineOfIndex(text, index) {
  let line = 1;
  for (let i = 0; i < index; i++) if (text[i] === "\n") line++;
  return line;
}

/**
 * Is the site at `index` test code? True when the file itself is a test
 * module by name, or when the site sits at or after the file's first
 * `#[cfg(test)]` attribute (the repo convention keeps test modules at the
 * end of the file).
 */
function isTestSite(relPath, stripped, index) {
  const base = path.basename(relPath);
  if (base === "tests.rs" || base.endsWith("_tests.rs")) return true;
  if (relPath.includes("/tests/")) return true;
  const cfgTest = stripped.indexOf("#[cfg(test)]");
  return cfgTest !== -1 && index >= cfgTest;
}

/** Scan the workspace once; returns per-crate stats and the flat site list. */
function scanWorkspace() {
  const crates = [];
  const allSites = [];
  let offsetReadTotal = 0;
  const offsetReadCrates = new Set();
  for (const crate of discoverCrates()) {
    const counts = Object.fromEntries(MEMBERS.map((member) => [member, 0]));
    let testSites = 0;
    let total = 0;
    for (const file of walkRustFiles(crate.srcDir)) {
      const relPath = path.relative(repoRoot, file).split(path.sep).join("/");
      const stripped = stripRust(readFileSync(file, "utf8"));
      for (const site of scanMembers(stripped, MEMBERS)) {
        counts[site.member] += 1;
        total += 1;
        const test = isTestSite(relPath, stripped, site.index);
        if (test) testSites += 1;
        allSites.push({
          crate: crate.name,
          relPath,
          line: lineOfIndex(stripped, site.index),
          member: site.member,
          test,
        });
      }
      const offsetReads = scanMembers(stripped, OFFSET_MEMBERS).length;
      if (offsetReads > 0) {
        offsetReadTotal += offsetReads;
        offsetReadCrates.add(crate.name);
      }
    }
    if (total > 0) crates.push({ name: crate.name, counts, total, testSites });
  }
  return { crates, allSites, offsetReadTotal, offsetReadCrateCount: offsetReadCrates.size };
}

/** `path:line` of the first inventoried site in the file, or throw. */
function citeSite(allSites, relPath, member = null) {
  const site = allSites.find(
    (s) => s.relPath === relPath && (member === null || s.member === member),
  );
  if (!site) {
    throw new Error(
      `citation lost: no ${member ?? "inventoried"} read found in ${relPath}; ` +
        `re-classify this consumer in tools/davinci/sourcelocation-inventory.mjs`,
    );
  }
  return `${site.relPath}:${site.line}`;
}

/** `path:line` of the first line matching `anchor` in the file, or throw. */
function citeAnchor(relPath, anchor) {
  const file = path.join(repoRoot, relPath);
  if (!existsSync(file)) {
    throw new Error(
      `citation lost: ${relPath} does not exist; ` +
        `re-classify this consumer in tools/davinci/sourcelocation-inventory.mjs`,
    );
  }
  const lines = readFileSync(file, "utf8").split("\n");
  const idx = lines.findIndex((line) => anchor.test(line));
  if (idx === -1) {
    throw new Error(
      `citation lost: ${anchor} not found in ${relPath}; ` +
        `re-classify this consumer in tools/davinci/sourcelocation-inventory.mjs`,
    );
  }
  return `${relPath}:${idx + 1}`;
}

/** References to `name` outside its defining module chain (stripped text). */
function countExternalReferences(name, excludeRelPaths) {
  let count = 0;
  for (const crate of discoverCrates()) {
    for (const file of walkRustFiles(crate.srcDir)) {
      const relPath = path.relative(repoRoot, file).split(path.sep).join("/");
      if (excludeRelPaths.includes(relPath)) continue;
      const stripped = stripRust(readFileSync(file, "utf8"));
      const re = new RegExp(`(^|[^A-Za-z0-9_])${name}([^A-Za-z0-9_]|$)`, "g");
      count += (stripped.match(re) ?? []).length;
    }
  }
  return count;
}

function generate() {
  const { crates, allSites, offsetReadTotal, offsetReadCrateCount } = scanWorkspace();
  const grandTotal = crates.reduce((sum, c) => sum + c.total, 0);
  const grandTest = crates.reduce((sum, c) => sum + c.testSites, 0);
  const memberTotals = Object.fromEntries(
    MEMBERS.map((member) => [member, crates.reduce((sum, c) => sum + c.counts[member], 0)]),
  );
  const sourceTotal = memberTotals.source;
  const lineColTotal = grandTotal - sourceTotal;

  const sorted = [...crates].sort((a, b) =>
    a.total !== b.total ? b.total - a.total : a.name < b.name ? -1 : 1,
  );

  const countsTable = formatTable(
    ["crate", ...MEMBERS.map((m) => `\`${m}\``), "total", "in test code"],
    ["left", ...MEMBERS.map(() => "right"), "right", "right"],
    [
      ...sorted.map((crate) => [
        `\`${crate.name}\``,
        ...MEMBERS.map((member) => String(crate.counts[member])),
        String(crate.total),
        String(crate.testSites),
      ]),
      [
        "**total**",
        ...MEMBERS.map((member) => String(memberTotals[member])),
        String(grandTotal),
        String(grandTest),
      ],
    ],
  );

  const lineColSites = allSites.filter((site) => site.member !== "source");
  const lineColTable = formatTable(
    ["member", "site", "test code"],
    ["left", "left", "left"],
    lineColSites.map((site) => [
      `\`${site.member}\``,
      `\`${site.relPath}:${site.line}\``,
      site.test ? "yes" : "no",
    ]),
  );

  // Group 1 citations: content reads that become `Span::slice`.
  const g1Codegen = citeSite(
    allSites,
    "crates/vize_atelier_core/src/codegen/expression/generate.rs",
  );
  const g1Croquis = citeSite(allSites, "crates/vize_croquis/src/drawer/template/components.rs");
  const g1Patina = citeSite(allSites, "crates/vize_patina/src/rules/script/template_scan.rs");
  const g1Vapor = citeSite(allSites, "crates/vize_atelier_vapor/src/transforms/v_on.rs");

  // Group 2 citations: line/column consumers that become offset-derived.
  const g2Comment = citeSite(
    allSites,
    "crates/vize_armature/src/parser/element/comment.rs",
    "start.line",
  );
  const g2Convert = citeAnchor(
    "crates/vize_maestro/src/utils/position.rs",
    /fn source_location_to_range/,
  );
  const g2ReliefTest = citeSite(allSites, "crates/vize_relief/src/relief/tests.rs", "start.line");
  const g2LineIndex = citeAnchor("crates/vize_carton/src/line_index.rs", /pub struct LineIndex/);

  // Group 3 citations: structures that delete outright.
  const g3Convert = g2Convert;
  const convertCallers = countExternalReferences("source_location_to_range", [
    "crates/vize_maestro/src/utils.rs",
    "crates/vize_maestro/src/utils/position.rs",
  ]);
  const g3Mapper = citeAnchor("crates/vize_atelier_jsx/src/span.rs", /pub struct SpanMapper/);
  const g3Stub = citeAnchor(
    "crates/vize_relief/src/relief/core.rs",
    /static STUB_LOCATION: SourceLocation/,
  );
  const g3Simple = citeAnchor(
    "crates/vize_relief/src/relief/expressions.rs",
    /pub struct SimpleExpressionNode/,
  );

  return `<!-- GENERATED FILE — do not edit by hand.
     Regenerate: ${REGEN_COMMAND}
     Verify:     node tools/davinci/sourcelocation-inventory.mjs --check
     Generator:  tools/davinci/sourcelocation-inventory.mjs -->

# \`SourceLocation\` consumer inventory

Every textual read of the relief \`SourceLocation\` members that
\`vize_carton::Span\` (\`crates/vize_carton/src/span.rs\`) deletes —
\`source\`, \`start.line\`, \`start.column\`, \`end.line\`, \`end.column\` —
across \`crates/*/src\`, plus the migration group each consumer moves to when
relief nodes carry two-u32 byte spans instead of owned
\`{ start: Position, end: Position, source: String }\` triples
([architecture.md](../architecture.md), S0). This is the migration map Davinci
P1 executes (P0-9).

## Resolution method (and its limits)

- Comments and string literals are stripped first
  (\`tools/davinci/lib/rust-source.mjs\`); member paths are then counted
  **textually** on loc-shaped receivers only: chained \`.loc.<member>\`,
  \`.loc().<member>\`, \`.location.<member>\` accesses, and bare locals named
  \`loc\` / \`location\` / \`*_loc\` / \`*_location\`.
- Reads of \`start.offset\` / \`end.offset\` are **not** inventoried: they
  survive the migration verbatim as \`span.start\` / \`span.end\`
  (${offsetReadTotal} loc-shaped offset-read sites across
  ${offsetReadCrateCount} crates at generation time).
- \`#[cfg(test)]\` code inside \`src/\` is included and reported in the
  "in test code" column: a site counts as test code when its file is a test
  module by name (\`tests.rs\`, \`*_tests.rs\`, \`/tests/\`) or sits at or
  after the file's first \`#[cfg(test)]\` attribute.
- The scan is not type-resolved. Known imprecision, spot-checked:
  - \`vize_doctor\` defines a namesake path+offset \`SourceLocation\`
    (\`crates/vize_doctor/src/model/evidence.rs\`) that is already
    span-shaped; its members (\`path\`, \`start: u32\`, \`end: u32\`) never
    form any of the five member paths, so it cannot contribute rows here and
    needs no migration.
  - \`BlockLocation\` (\`crates/vize_atelier_sfc/src/types.rs\`) also sits
    behind \`loc\` fields, but its \`start\`/\`end\` are plain \`usize\`
    offsets with flat \`start_line\`/\`start_column\` siblings — none of the
    five member paths exist on it, so it cannot collide either.
  - Locals bound under non-loc names are missed:
    \`crates/vize_relief/src/relief/tests.rs\` binds \`SourceLocation::STUB\`
    as \`stub\` and reads \`stub.start.line\` / \`stub.start.column\`
    (2 sites, test-only).
  - \`Position\` values escaping whole are counted where the \`.line\` /
    \`.column\` read happens, which a loc-shaped filter can miss: the two
    reads inside \`internal_to_lsp_position\`
    (\`crates/vize_maestro/src/utils/position.rs\`, receiver \`pos\`) are the
    only such sites today, reached only through \`source_location_to_range\`
    (${convertCallers} references outside its own module at generation time).

## Reads per crate × member

${countsTable}
## Every \`line\` / \`column\` read site

The line/column members are read so rarely that the sites fit in one table:

${lineColTable}
## Migration groups

### Group 1 — content reads move to \`Span::slice\` (${sourceTotal} sites, every \`source\` read)

The dominant consumer class by far: code that wants **the text a node
covers** — codegen re-emitting an expression, croquis capturing a binding
name, a lint rule inspecting raw expression text, a test asserting what the
parser captured. Today each read pays for an owned \`String\` copied into the
node at parse time; with spans the node stores 8 bytes and the read becomes
\`span.slice(source)\` against the one authored source string (or, for
block-relative spans, against that block's text). Representative sites:

- \`${g1Codegen}\` — codegen emits the recorded expression text verbatim
- \`${g1Croquis}\` — croquis captures component/expression text into
  analysis products
- \`${g1Patina}\` — lint rule matches against raw expression text
- \`${g1Vapor}\` — vapor transform re-wraps an expression by copying
  \`loc.source\` into a new node (see also group 3: the copy itself
  disappears)

### Group 2 — line/column reads move to offset-derived rendering (${lineColTotal} direct sites + 4 known-missed, see limits)

Line/column exist only at diagnostic- or LSP-rendering time under Davinci:
derived from byte offsets via \`vize_carton::line_index::LineIndex\`
(\`${g2LineIndex}\`) at the edge that needs them — exactly how the
source-map \`finish()\` step and Patina's output layer already work. The
eagerly-stored \`Position { line, column }\` pairs these sites read today
delete with the type:

- \`${g2Comment}\` — the **only** production read: seeds
  \`parse_vize_directive\` with the comment's start line; becomes a
  \`LineIndex\` lookup from the span start (or a plain offset if the
  consumer is offset-ready)
- \`${g2Convert}\` — \`source_location_to_range\` converts stored
  \`Position\`s to LSP positions via \`internal_to_lsp_position\` (2 of the
  4 known-missed reads: \`pos.line\` / \`pos.column\`; the other 2 are the
  \`stub\` locals in the relief tests); its replacement
  is the offset → \`LineIndex\` path every live maestro collector already
  uses — and it has no callers, so see group 3
- \`${g2ReliefTest}\` — relief tests pinning the 1-indexed line/column
  convention of stub locations; re-target to span offsets (the 1-based
  convention itself becomes a rendering-layer concern)

### Group 3 — delete outright

Structures whose entire job is carrying or converting the eager
representation. No slice or line/col replacement — they stop existing:

- \`${g3Convert}\` — \`source_location_to_range\` has
  ${convertCallers} references outside its own module; dead conversion code
- \`${g3Mapper}\` — \`SpanMapper\` eagerly expands oxc byte spans
  (\`oxc_span::Span\`, already \`{ start: u32, end: u32 }\`) into owned
  \`SourceLocation\`s for every lowered JSX node; under S0 the oxc span
  passes through unchanged and the whole conversion layer deletes
- \`${g3Stub}\` — \`STUB_LOCATION\` / \`SourceLocation::STUB\`: the
  owned-string stub machinery for generated nodes collapses to
  \`Span::new(0, 0)\`
- \`${g3Simple}\` — \`SimpleExpressionNode\` stores \`content: String\`
  **and** \`loc.source\` duplicating it; span-carrying nodes keep one span,
  and every group-1 site that today clones \`loc.source\` to build another
  node (e.g. \`${g1Vapor}\`) stops copying text entirely
`;
}

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check") {
    console.error("usage: node tools/davinci/sourcelocation-inventory.mjs --write | --check");
    process.exit(2);
  }
  const generated = generate();
  if (mode === "--write") {
    writeFileSync(ARTIFACT, generated);
    console.log(`wrote ${ARTIFACT_REL}`);
    return;
  }
  // --check
  if (!existsSync(ARTIFACT)) {
    console.error(`stale: ${ARTIFACT_REL} does not exist. Regenerate with: ${REGEN_COMMAND}`);
    process.exit(1);
  }
  const committed = readFileSync(ARTIFACT, "utf8");
  if (committed === generated) {
    console.log(`${ARTIFACT_REL} is up to date`);
    return;
  }
  const committedLines = committed.split("\n");
  const generatedLines = generated.split("\n");
  let firstDiff = -1;
  const max = Math.max(committedLines.length, generatedLines.length);
  for (let i = 0; i < max; i++) {
    if (committedLines[i] !== generatedLines[i]) {
      firstDiff = i;
      break;
    }
  }
  console.error(`stale: ${ARTIFACT_REL} drifted from the current sources.`);
  console.error(
    `  first differing line: ${firstDiff + 1} (committed ${committedLines.length} lines, regenerated ${generatedLines.length})`,
  );
  if (firstDiff >= 0) {
    console.error(`  - ${(committedLines[firstDiff] ?? "<eof>").slice(0, 160)}`);
    console.error(`  + ${(generatedLines[firstDiff] ?? "<eof>").slice(0, 160)}`);
  }
  console.error(`  Regenerate with: ${REGEN_COMMAND}`);
  process.exit(1);
}

main();
