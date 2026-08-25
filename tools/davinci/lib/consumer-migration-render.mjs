// Markdown renderer for the generated consumer migration surface inventory.

import { formatTable } from "./markdown.mjs";
import { SURFACES, surfaceNameKind } from "./consumer-migration-scan.mjs";

function n(value) {
  return String(value);
}

function surfaceList(surfaceCounts) {
  const labels = SURFACES.filter((surface) => surfaceCounts[surface.id] > 0).map(
    (surface) => `${surface.label} ${surfaceCounts[surface.id]}`,
  );
  return labels.length === 0 ? "-" : labels.join("<br>");
}

function modeLabel(mode) {
  if (mode === "manifest") return "manifest";
  if (mode === "test") return "test/dev";
  return "source";
}

function matchedNamesLabel(surface) {
  const classified = new Set([...(surface.preferredNames ?? []), ...(surface.compatNames ?? [])]);
  const rows = [];
  if (surface.preferredNames?.length) {
    rows.push(`preferred: ${surface.preferredNames.map((name) => `\`${name}\``).join(", ")}`);
  }
  if (surface.compatNames?.length) {
    rows.push(`compat/code-name: ${surface.compatNames.map((name) => `\`${name}\``).join(", ")}`);
  }
  const otherNames = surface.names.filter((name) => !classified.has(name));
  if (otherNames.length > 0) {
    const label = surface.group === "raw" ? "raw" : "legacy";
    rows.push(`${label}: ${otherNames.map((name) => `\`${name}\``).join(", ")}`);
  }
  return rows.join("<br>");
}

function renderSurfaceCounts(consumer) {
  const rows = SURFACES.map((surface) => [
    surface.label,
    n(consumer.surfaceCounts[surface.id]),
    n(
      consumer.fileRows
        .filter((row) => row.mode !== "test")
        .reduce((sum, row) => sum + row.surfaceCounts[surface.id], 0),
    ),
    n(
      consumer.fileRows
        .filter((row) => row.mode === "test")
        .reduce((sum, row) => sum + row.surfaceCounts[surface.id], 0),
    ),
  ]).filter((row) => row[1] !== "0");
  if (rows.length === 0) return "_No direct surface mentions found._\n";
  return formatTable(
    ["surface", "total sites", "source/manifest", "test/dev"],
    ["left", "right", "right", "right"],
    rows,
  );
}

function renderFileRows(consumer, modePredicate) {
  const rows = consumer.fileRows
    .filter(modePredicate)
    .sort((a, b) => b.total - a.total || a.relPath.localeCompare(b.relPath))
    .slice(0, 5)
    .map((row) => [
      `\`${row.relPath}:${row.firstLine}\``,
      modeLabel(row.mode),
      surfaceList(row.surfaceCounts),
      n(row.total),
    ]);
  if (rows.length === 0) return "_No files in this class._\n";
  return formatTable(
    ["file", "class", "surfaces", "sites"],
    ["left", "left", "left", "right"],
    rows,
  );
}

function renderSummary(consumers) {
  return formatTable(
    [
      "consumer",
      "stage/Davinci",
      "preferred stage names",
      "compat code names",
      "old AST/Croquis",
      "raw OXC",
      "source/manifest",
      "test/dev",
      "surface files",
      "scanned files",
    ],
    ["left", "right", "right", "right", "right", "right", "right", "right", "right", "right"],
    consumers.map((consumer) => [
      consumer.label,
      n(consumer.groupCounts.stage),
      n(consumer.nameKindCounts.preferred),
      n(consumer.nameKindCounts.compat),
      n(consumer.groupCounts.old),
      n(consumer.groupCounts.raw),
      n(consumer.modeCounts.source + consumer.modeCounts.manifest),
      n(consumer.modeCounts.test),
      n(consumer.surfaceFileCount),
      n(consumer.fileCount),
    ]),
  );
}

function renderConsumer(consumer) {
  const sourceRows = consumer.fileRows.filter((row) => row.mode !== "test").length;
  const testRows = consumer.fileRows.filter((row) => row.mode === "test").length;
  const omittedSource = Math.max(0, sourceRows - 5);
  const omittedTest = Math.max(0, testRows - 5);

  return (
    `### ${consumer.label}

Scope: ${consumer.scope}. This is a lexical inventory, not a rollout gate.

${renderSurfaceCounts(consumer)}
#### Top source and manifest files

${renderFileRows(consumer, (row) => row.mode !== "test")}
${omittedSource > 0 ? `Additional source/manifest rows are in the TSV: ${omittedSource} omitted.\n` : ""}
#### Top test/dev files

${renderFileRows(consumer, (row) => row.mode === "test")}
${omittedTest > 0 ? `Additional test/dev rows are in the TSV: ${omittedTest} omitted.\n` : ""}`.trimEnd() +
    "\n"
  );
}

function renderSlices() {
  return `## Independently mergeable no-rollout slices

1. \`test(davinci): pin consumer migration surfaces\` - this artifact and its
   drift test. It makes the current dependency shape reviewable without
   changing command routing or defaults.
2. \`refactor(compiler): introduce stage-named compiler boundary adapters\` -
   add S0/S1/S2 adapter entrypoints inside the atelier crates while continuing
   to feed the existing Relief/Croquis pipeline. Guard with compiler fixture
   parity and keep the \`vize build\` path unchanged.
3. \`refactor(linter): add template analysis facade\` - move rule code toward
   a stable analysis contract while the facade is still backed by
   Relief/Croquis. Guard with lint divergence and rule fixture snapshots; no
   default linter backend switch.
4. \`refactor(typechecker): add virtual document boundary\` - introduce a
   narrow S0/S1 input contract for virtual TS generation and adapt current
   callers into it. Guard with the existing typecheck fixture matrix and
   real-project rows.
5. \`test(content-mapper): pin stage-neutral mapping protocol fixtures\` -
   expand content-mapper protocol fixtures around spans, virtual extensions,
   package routes, and declaration-map lookups. Keep the external tsgo protocol
   byte-compatible.
6. \`refactor(formatter): isolate region formatting plan\` - keep Glyph/OXC
   output unchanged, but put region extraction and script formatting behind a
   stage-neutral formatting plan. Guard with idempotence and range-formatting
   fixtures.
7. \`refactor(lsp): add current-backend adapter boundary\` - route Maestro
   document/virtual-code feature inputs through a backend trait whose first
   implementation delegates to the current Armature/Croquis/Canon stack. Guard
   hover, definition, diagnostics, semantic tokens, and formatting with
   existing LSP e2e tests.
8. \`refactor(davinci): align physical layer names with s0/s1/s2\` - migrate
   public internal module/crate references toward S0/S1/S2 naming in small
   aliasing steps. Keep code names only as compatibility aliases until all
   consumers have moved.

Rollout remains explicitly out of scope for these slices: none should switch
user-visible defaults, command dispatch, package exports, editor activation, or
protocol behavior.`;
}

export function renderConsumerMigrationSurfaces(scan, options) {
  const surfaceLegend = formatTable(
    ["surface", "group", "matched name classes"],
    ["left", "left", "left"],
    scan.surfaces.map((surface) => [surface.label, surface.group, matchedNamesLabel(surface)]),
  );

  return `<!-- GENERATED FILE - do not edit by hand.
     Regenerate: ${options.regenCommand}
     Verify:     node tools/davinci/consumer-migration-surfaces.mjs --check
     Generator:  tools/davinci/consumer-migration-surfaces.mjs -->

# Consumer migration surfaces

This inventory records where the user-facing consumers that must eventually
sit on Davinci/S0/S1/S2 still name stage crates, legacy AST/parser/Croquis
crates, or raw OXC crates directly on current \`origin/main\`. It is an
observational guard for planning only. It does not change rollout state.

## Resolution method

- Rust comments and string literals are stripped before matching; Cargo
  comments are stripped while dependency keys remain visible.
- Matches are lexical crate/surface names, not type-resolved imports. A row
  means "this file directly names this surface", not necessarily that every
  mention is a runtime dependency edge.
- Stage names are split into preferred physical names and compatibility
  code-name aliases so S0/S1/S2 migration work is measurable without changing
  rollout state.
- \`source/manifest\` includes production Rust files plus crate manifests.
  \`test/dev\` includes crate \`tests\`, \`benches\`, \`tests.rs\`,
  \`*_tests.rs\`, and Rust sites after the first \`#[cfg(test)]\` in a file.
- Content-mapper files under Canon are reported separately from the broader
  typechecker row so that protocol work can move in smaller PRs.
- Full file x surface x matched-name rows are generated in \`${options.rowsRel}\`; this
  markdown keeps only top impact files to stay under the source-length gate.

## Surface legend

${surfaceLegend}
## Consumer summary

${renderSummary(scan.consumers)}
## Consumer details

${scan.consumers.map(renderConsumer).join("\n")}
${renderSlices()}

## Regeneration

\`\`\`sh
${options.regenCommand}
node tools/davinci/consumer-migration-surfaces.mjs --check
\`\`\`
`;
}

export function renderConsumerMigrationSurfaceRows(scan) {
  const lines = [
    [
      "consumer_id",
      "consumer",
      "class",
      "file",
      "first_line",
      "surface_id",
      "surface",
      "surface_group",
      "matched_name",
      "name_kind",
      "sites",
    ].join("\t"),
  ];
  for (const consumer of scan.consumers) {
    for (const row of consumer.fileRows) {
      for (const surface of SURFACES) {
        for (const name of surface.names) {
          const sites = row.surfaceNameCounts[surface.id][name] ?? 0;
          if (sites === 0) continue;
          lines.push(
            [
              consumer.id,
              consumer.label,
              modeLabel(row.mode),
              row.relPath,
              String(row.firstLine),
              surface.id,
              surface.label,
              surface.group,
              name,
              surfaceNameKind(surface, name),
              String(sites),
            ].join("\t"),
          );
        }
      }
    }
  }
  return `${lines.join("\n")}\n`;
}
