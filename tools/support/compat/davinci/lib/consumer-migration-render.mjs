// Renderer for the committed consumer migration surface inventory: a
// markdown index whose content depends only on the scan configuration
// (method, surface legend, consumer scopes, shard layout) plus one TSV shard
// per (consumer, crate). Nothing committed aggregates across files or crates,
// so two PRs touching different crates never edit the same committed file;
// totals live in the on-demand `--summary` view (consumer-migration-summary.mjs).

import { formatTable } from "./markdown.mjs";
import { SURFACES, surfaceNameKind } from "./consumer-migration-scan.mjs";

export const SHARD_DIR_REL = "davinci-road/plan/consumer-migration-surfaces";

export function surfaceList(surfaceCounts) {
  const labels = SURFACES.filter((surface) => surfaceCounts[surface.id] > 0).map(
    (surface) => `${surface.label} ${surfaceCounts[surface.id]}`,
  );
  return labels.length === 0 ? "-" : labels.join("<br>");
}

export function modeLabel(mode) {
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

// The crate a scanned file belongs to: every scope entry lives under `crates/<crate>/`.
function crateOf(relPath) {
  const [root, crate] = relPath.split("/");
  if (root !== "crates" || !crate) throw new Error(`scanned file outside crates/: ${relPath}`);
  return crate;
}

function entryRoot(entry) {
  return entry.crate ? `crates/${entry.crate}` : entry.path;
}

function entryLabel(entry) {
  const filtered = entry.include || entry.exclude ? " (filtered, see scope)" : "";
  if (entry.crate) {
    return `crate \`${entry.crate}\` (\`Cargo.toml\`, \`src\`, \`tests\`, \`benches\`)${filtered}`;
  }
  return `\`${entry.path}\`${filtered}`;
}

/** Every (consumer, crate) shard, fixed by the scan configuration alone. */
export function shardLayout(consumers) {
  const shards = [];
  for (const consumer of consumers) {
    const byCrate = new Map();
    for (const entry of consumer.entries) {
      const crate = crateOf(entryRoot(entry));
      if (!byCrate.has(crate)) byCrate.set(crate, []);
      byCrate.get(crate).push(entry);
    }
    for (const crate of [...byCrate.keys()].sort((a, b) => a.localeCompare(b))) {
      shards.push({
        consumer,
        crate,
        entries: byCrate.get(crate),
        relPath: `${SHARD_DIR_REL}/${consumer.id}/${crate}.tsv`,
      });
    }
  }
  return shards;
}

function renderShardTable(consumers) {
  return formatTable(
    ["consumer", "shard", "scanned roots"],
    ["left", "left", "left"],
    shardLayout(consumers).map((shard) => {
      const rel = shard.relPath.slice(SHARD_DIR_REL.length + 1);
      return [
        shard.consumer.label,
        `[\`${rel}\`](./consumer-migration-surfaces/${rel})`,
        shard.entries.map(entryLabel).join("<br>"),
      ];
    }),
  );
}

function renderScopes(consumers) {
  return consumers
    .map((consumer) => `- **${consumer.label}** (\`${consumer.id}\`): ${consumer.scope}.`)
    .join("\n");
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
     Verify:     rust-script tools/commands/davinci/consumer-migration-surfaces.rs --check
     Totals:     ${options.summaryCommand}
     Generator:  tools/support/compat/davinci/consumer-migration-surfaces.mjs -->

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

## Surface legend

${surfaceLegend}
## Consumers

${renderScopes(scan.consumers)}

## Shards

Every row lives in exactly one TSV shard per (consumer, crate) under
\`${SHARD_DIR_REL}/\`: one row per file x class x surface x matched name,
columns \`consumer_id\`, \`consumer\`, \`class\` (\`source\`, \`manifest\`,
\`test/dev\`), \`file\`, \`first_line\`, \`surface_id\`, \`surface\`,
\`surface_group\`, \`matched_name\`, \`name_kind\`, \`sites\`. The shard set is
fixed by the consumer scopes (table below), so it only changes when a scope does.

Cross-file aggregates — per-consumer and per-surface totals and the top files
by site count — are deliberately **not committed**: they changed with every PR
and made every open PR conflict. They are pure sums over the shards; print
them with \`${options.summaryCommand}\`. The staleness check (TS-12)
byte-compares this page, every shard, and the shard set itself.

${renderShardTable(scan.consumers)}
${renderSlices()}

## Regeneration

\`\`\`sh
${options.regenCommand}
rust-script tools/commands/davinci/consumer-migration-surfaces.rs --check
${options.summaryCommand}
\`\`\`
`;
}

const ROW_COLUMNS = [
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
];

function renderRows(consumer, fileRows) {
  const lines = [ROW_COLUMNS.join("\t")];
  for (const row of fileRows) {
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
  return `${lines.join("\n")}\n`;
}

/** One TSV per (consumer, crate) shard; a shard with no rows keeps its header. */
export function renderConsumerMigrationShards(scan) {
  const shards = shardLayout(scan.consumers);
  const known = new Set(shards.map((shard) => `${shard.consumer.id}\0${shard.crate}`));
  for (const consumer of scan.consumers) {
    for (const row of consumer.fileRows) {
      const key = `${consumer.id}\0${crateOf(row.relPath)}`;
      if (!known.has(key)) throw new Error(`no shard for ${consumer.id} row ${row.relPath}`);
    }
  }
  return shards.map((shard) => ({
    relPath: shard.relPath,
    text: renderRows(
      shard.consumer,
      shard.consumer.fileRows.filter((row) => crateOf(row.relPath) === shard.crate),
    ),
  }));
}

/** Every row of every consumer as one TSV (header + rows): the in-memory union of the shards. */
export function renderConsumerMigrationSurfaceRows(scan) {
  const rows = scan.consumers.flatMap((consumer) =>
    renderRows(consumer, consumer.fileRows).split("\n").slice(1, -1),
  );
  return `${[ROW_COLUMNS.join("\t"), ...rows].join("\n")}\n`;
}
