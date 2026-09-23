// On-demand `--summary` view of the consumer migration surface inventory:
// per-consumer totals, per-surface totals, and the top files by site count.
// These are cross-file aggregates, so they are never committed — every PR
// that touched any scanned file changed them and made every other open PR
// conflict. They are pure sums over the committed per-crate TSV shards.

import { formatTable } from "./markdown.mjs";
import { modeLabel, surfaceList } from "./consumer-migration-render.mjs";
import { SURFACES } from "./consumer-migration-scan.mjs";

export const SUMMARY_COMMAND =
  "rust-script tools/commands/davinci/consumer-migration-surfaces.rs --summary";

function n(value) {
  return String(value);
}

function renderSurfaceCounts(consumer) {
  const sumFor = (surface, predicate) =>
    consumer.fileRows
      .filter(predicate)
      .reduce((sum, row) => sum + row.surfaceCounts[surface.id], 0);
  const rows = SURFACES.map((surface) => [
    surface.label,
    n(consumer.surfaceCounts[surface.id]),
    n(sumFor(surface, (row) => row.mode !== "test")),
    n(sumFor(surface, (row) => row.mode === "test")),
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

function renderTotals(consumers) {
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
${omittedSource > 0 ? `Additional source/manifest rows are in the shards: ${omittedSource} omitted.\n` : ""}
#### Top test/dev files

${renderFileRows(consumer, (row) => row.mode === "test")}
${omittedTest > 0 ? `Additional test/dev rows are in the shards: ${omittedTest} omitted.\n` : ""}`.trimEnd() +
    "\n"
  );
}

/** The cross-file aggregates printed by `--summary`. Never committed. */
export function renderConsumerMigrationSummary(scan) {
  return `# Consumer migration surfaces summary (computed, not committed)

Cross-file aggregates of the sharded inventory in
\`docs/davinci/plan/consumer-migration-surfaces/\`. Printed by
\`${SUMMARY_COMMAND}\`; every number is a sum over the TSV shards.

## Consumer summary

${renderTotals(scan.consumers)}
## Consumer details

${scan.consumers.map(renderConsumer).join("\n")}`;
}
