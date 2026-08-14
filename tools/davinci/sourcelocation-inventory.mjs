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

import { formatTable } from "./lib/markdown.mjs";
import {
  MEMBERS,
  citeAnchor,
  citeSite,
  countExternalReferences,
  scanWorkspace,
} from "./lib/sourcelocation-scan.mjs";
import { repoRoot } from "./lib/paths.mjs";

const ARTIFACT_REL = "davinci-road/plan/sourcelocation-inventory.md";
const ARTIFACT = path.join(repoRoot, ARTIFACT_REL);
const REGEN_COMMAND = "node tools/davinci/sourcelocation-inventory.mjs --write";

/** The five member paths `Span` deletes, in table-column order. */

/** Offset member paths that survive the migration as `span.start`/`span.end`. */

/** Is `ident` a loc-shaped receiver or field name? */
/** `source` reads counted at generation time of the P0-9 map, all migrated
 * to `Span::slice` by Davinci P1-3. */
const P0_9_SOURCE_READS = 106;

function generate() {
  const { crates, allSites, offsetReadTotal, offsetReadCrateCount } = scanWorkspace();
  const grandTotal = crates.reduce((sum, c) => sum + c.total, 0);
  const grandTest = crates.reduce((sum, c) => sum + c.testSites, 0);
  const memberTotals = Object.fromEntries(
    MEMBERS.map((member) => [member, crates.reduce((sum, c) => sum + c.counts[member], 0)]),
  );
  const sourceTotal = memberTotals.source;
  if (sourceTotal !== 0) {
    const site = allSites.find((s) => s.member === "source");
    throw new Error(
      `P1-3 ratchet: ${sourceTotal} \`.source\` read(s) reintroduced on loc-shaped receivers ` +
        `(first: ${site.relPath}:${site.line}); read covered text via \`span.slice(source)\` instead`,
    );
  }
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

  // Group 1 citations: content reads, now anchored on their migrated
  // `Span::slice` forms so a moved consumer still fails regeneration.
  const g1Codegen = citeAnchor(
    "crates/vize_atelier_core/src/codegen/expression/generate.rs",
    /span\.slice\(&ctx\.source\)/,
  );
  const g1Croquis = citeAnchor(
    "crates/vize_croquis/src/drawer/template/components.rs",
    /span\.slice\(&self\.template_source\)/,
  );
  const g1Patina = citeAnchor(
    "crates/vize_patina/src/rules/script/template_scan.rs",
    /span\.slice\(source\)/,
  );
  const g1Vapor = citeAnchor(
    "crates/vize_atelier_vapor/src/transforms/v_on.rs",
    /span\.slice\(source\)/,
  );

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
across \`crates/*/src\`, plus the migration group each consumer moves to as
relief nodes switch to two-u32 byte spans instead of owned
\`{ start: Position, end: Position, source: String }\` triples
([architecture.md](../architecture.md), S0). This is the migration map Davinci
P1 executes (P0-9). P1-3 executed group 1: every \`source\` read is migrated,
the scan now counts zero, and regeneration fails if one comes back. The
line/column members remain until P1-4.

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

### Group 1 — content reads moved to \`Span::slice\` (migrated by P1-3: ${P0_9_SOURCE_READS} sites at P0-9, ${sourceTotal} remain)

The dominant consumer class by far: code that wants **the text a node
covers** — codegen re-emitting an expression, croquis capturing a binding
name, a lint rule inspecting raw expression text, a test asserting what the
parser captured. Each read used to pay for an owned \`String\` copied into
the node at parse time; the node now stores 8 bytes and the read is
\`span.slice(source)\` against the one authored source string (or, for
block-relative spans, against that block's text). Representative migrated
sites:

- \`${g1Codegen}\` — codegen reads the recorded expression text verbatim
- \`${g1Croquis}\` — croquis captures component/expression text into
  analysis products
- \`${g1Patina}\` — lint rule matches against raw expression text
- \`${g1Vapor}\` — vapor transform re-wraps an expression from the covered
  text (the owned copy the pre-span node stored is gone; see also group 3)

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
- \`${g3Simple}\` — \`SimpleExpressionNode\` stored \`content: String\`
  **and** \`loc.source\` duplicating it; since P1-3 the node keeps one span
  next to \`content\`, and every group-1 site that cloned \`loc.source\` to
  build another node (e.g. \`${g1Vapor}\`) slices on demand instead
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
