#!/usr/bin/env node
// SourceLocation consumer inventory (Davinci P0-9).
//
// Counts every textual read of the relief `SourceLocation` members that the
// Davinci `Span` type (crates/vize_carton/src/span.rs) deleted —
// `source`, `start.line`, `start.column`, `end.line`, `end.column` — across
// `crates/*/src/**/*.rs`, grouped by crate and member, and emits the
// migration record as davinci-road/plan/sourcelocation-inventory.md. The
// migration is fully executed (P1-3 retired `source`, P1-4 retired
// line/column with the `Position` type itself), so the scan doubles as the
// ratchet: any member read coming back fails regeneration, as does any
// reappearance of the deleted carriers.
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
  assertAnchorAbsent,
  assertSymbolAbsent,
  citeAnchor,
  scanWorkspace,
} from "./lib/sourcelocation-scan.mjs";
import { repoRoot } from "./lib/paths.mjs";

const ARTIFACT_REL = "davinci-road/plan/sourcelocation-inventory.md";
const ARTIFACT = path.join(repoRoot, ARTIFACT_REL);
const REGEN_COMMAND = "node tools/davinci/sourcelocation-inventory.mjs --write";

/** `source` reads counted at generation time of the P0-9 map, all migrated
 * to `Span::slice` by Davinci P1-3. */
const P0_9_SOURCE_READS = 106;
/** Line/column reads counted when P1-4 executed: 3 direct sites plus the 4
 * known-missed reads the limits section of the P0-9 map documented. */
const P1_4_LINE_COL_READS = "3 direct sites + 4 known-missed";

function generate() {
  const { crates, allSites, offsetReadTotal, offsetReadCrateCount } = scanWorkspace();
  const grandTotal = crates.reduce((sum, c) => sum + c.total, 0);
  if (grandTotal !== 0) {
    const site = allSites[0];
    throw new Error(
      `P1-3/P1-4 ratchet: ${grandTotal} deleted-member read(s) reintroduced on loc-shaped ` +
        `receivers (first: \`${site.member}\` at ${site.relPath}:${site.line}); read covered ` +
        `text via \`span.slice(source)\` and derive line/column from offsets via ` +
        `\`vize_carton::line_index\` instead`,
    );
  }

  // The deleted carriers must stay deleted (P1-4).
  assertAnchorAbsent(
    "crates/vize_relief/src/relief/core.rs",
    /pub struct Position/,
    "the eager `Position { offset, line, column }` type",
  );
  assertSymbolAbsent(
    "source_location_to_range",
    "maestro's stored-Position -> LSP Range converter",
  );
  assertSymbolAbsent(
    "internal_to_lsp_position",
    "maestro's stored-Position -> LSP Position converter",
  );

  const countsTable = formatTable(
    ["crate", ...MEMBERS.map((m) => `\`${m}\``), "total", "in test code"],
    ["left", ...MEMBERS.map(() => "right"), "right", "right"],
    [["**total**", ...MEMBERS.map(() => "0"), "0", "0"]],
  );

  // Group 1 citations: content reads, anchored on their migrated
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

  // Group 2 citations: line/column consumers, anchored on their migrated
  // offset-derived forms.
  const g2Comment = citeAnchor(
    "crates/vize_armature/src/parser/element/comment.rs",
    /parse_vize_directive\(content, 1, loc\.span\.start\)/,
  );
  const g2RenderedPosition = citeAnchor(
    "crates/vize_relief/src/errors/render.rs",
    /struct RenderedPosition/,
  );
  const g2LineIndex = citeAnchor("crates/vize_carton/src/line_index.rs", /pub struct LineIndex/);
  const g2ReliefTest = citeAnchor(
    "crates/vize_relief/src/relief/tests.rs",
    /fn source_location_stub/,
  );

  // Group 3 citations: what remains of the deleted carriers.
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
\`vize_carton::Span\` (\`crates/vize_carton/src/span.rs\`) deleted —
\`source\`, \`start.line\`, \`start.column\`, \`end.line\`, \`end.column\` —
across \`crates/*/src\`, plus the migration group each consumer moved to as
relief nodes switched to two-u32 byte spans instead of owned
\`{ start: Position, end: Position, source: String }\` triples
([architecture.md](../architecture.md), S0). This was the migration map
Davinci P1 executed (P0-9). P1-3 executed group 1 (\`source\` reads to
\`Span::slice\`); P1-4 executed groups 2 and 3 (line/column reads to
offset-derived rendering, the \`Position\` type and its converters deleted).
The scan now counts zero across all five members, and regeneration fails if
any read — or any deleted carrier — comes back.

## Resolution method (and its limits)

- Comments and string literals are stripped first
  (\`tools/davinci/lib/rust-source.mjs\`); member paths are then counted
  **textually** on loc-shaped receivers only: chained \`.loc.<member>\`,
  \`.loc().<member>\`, \`.location.<member>\` accesses, and bare locals named
  \`loc\` / \`location\` / \`*_loc\` / \`*_location\`.
- Reads of \`span.start\` / \`span.end\` are **not** inventoried: they are
  the surviving offset representation — the pre-migration
  \`start.offset\` / \`end.offset\` reads moved to them verbatim
  (${offsetReadTotal} loc-shaped span-read sites across
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
    needed no migration.
  - \`BlockLocation\` (\`crates/vize_atelier_sfc/src/types.rs\`) also sits
    behind \`loc\` fields, but its \`start\`/\`end\` are plain \`usize\`
    offsets with flat \`start_line\`/\`start_column\` siblings — none of the
    five member paths exist on it, so it cannot collide either.

## Reads per crate × member

${countsTable}
## Migration groups

### Group 1 — content reads moved to \`Span::slice\` (migrated by P1-3: ${P0_9_SOURCE_READS} sites at P0-9, 0 remain)

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

### Group 2 — line/column reads moved to offset-derived rendering (migrated by P1-4: ${P1_4_LINE_COL_READS} at P0-9, 0 remain)

Line/column exist only at diagnostic- or LSP-rendering time under Davinci:
derived from byte offsets via \`vize_carton::line_index::LineIndex\`
(\`${g2LineIndex}\`) at the edge that needs them — exactly how the
source-map \`finish()\` step and Patina's output layer already worked. The
eagerly-stored \`Position { line, column }\` pairs deleted with the type.
Where each read went:

- \`${g2Comment}\` — the only production read seeded
  \`parse_vize_directive\` with the comment's start line, which that caller
  discards (only the directive kind survives); it now passes the constant
  line the retired tracking always reported
- \`${g2RenderedPosition}\` — the one output path that printed stored
  line/column (the SFC gate / binding-boundary debug rendering) now derives
  display coordinates from the rendered source text via \`LineIndex\`. This
  keeps \`SourceLocation\` span-only while making multiline diagnostics point
  at the actual line and column instead of the retired frozen
  \`line: 1, column: offset + 1\` approximation
- \`crates/vize_maestro/src/utils/position.rs\` — \`source_location_to_range\`
  / \`internal_to_lsp_position\` converted stored \`Position\`s to LSP
  positions; they had no callers and are deleted (regeneration asserts they
  stay gone)
- \`${g2ReliefTest}\` — relief tests that pinned the 1-indexed line/column
  convention of stub locations now pin span offsets (the 1-based convention
  itself is a rendering-layer concern)

### Group 3 — deleted outright

Structures whose entire job was carrying or converting the eager
representation:

- \`crates/vize_relief/src/relief/core.rs\` — the \`Position\` type itself
  is gone; \`SourceLocation\` is the 8-byte \`{ span: Span }\` (regeneration
  asserts \`pub struct Position\` stays gone from the module)
- \`${g3Mapper}\` — \`SpanMapper\` no longer expands oxc byte spans into
  eager positions; \`location()\` is a direct offset carry-over and the
  \`LineIndex\` it built per module is deleted
- \`${g3Stub}\` — \`STUB_LOCATION\` / \`SourceLocation::STUB\` collapsed to
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
