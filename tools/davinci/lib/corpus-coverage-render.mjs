// Markdown rendering for the corpus construct-coverage report (Davinci P0-6).
// Tables go through the shared vp-canonical `formatTable` helper so the
// committed artifact byte-matches what `vp check` would format.

import { formatTable } from "./markdown.mjs";

/** `formatTable` without its trailing newline: the report joins its own lines. */
function table(header, alignRight, rows) {
  const aligns = alignRight.map((right) => (right ? "right" : "left"));
  return formatTable(
    header,
    aligns,
    rows.map((row) => row.map(String)),
  ).replace(/\n$/, "");
}

function dimensionTable(title, ids, hydrated, pick) {
  const header = ["project", ...ids];
  const alignRight = [false, ...ids.map(() => true)];
  const rows = hydrated.map((project) => [
    `\`${project.id}\``,
    ...ids.map((id) => pick(project.counts, id)),
  ]);
  const totals = ids.map((id) =>
    hydrated.reduce((sum, project) => sum + pick(project.counts, id), 0),
  );
  const seen = ids.map((id) => hydrated.filter((project) => pick(project.counts, id) > 0).length);
  rows.push(["**total sites**", ...totals]);
  rows.push(["**projects seen**", ...seen]);
  return `### ${title}\n\n${table(header, alignRight, rows)}`;
}

const DIMENSIONS = [
  {
    title: "Dimension 1: element_kind (start-tag classes)",
    ids: (taxonomy) => taxonomy.element_kind.map((entry) => entry.id),
    pick: (counts, id) => counts.elementKind[id],
  },
  {
    title: "Dimension 2: directive (attribute names, incl. `:` / `@` shorthand)",
    ids: (taxonomy) => taxonomy.directive.map((entry) => entry.id),
    pick: (counts, id) => counts.directive[id],
  },
  {
    title: "Dimension 3: modifier_class (modifier tokens on the applicable directive)",
    ids: (taxonomy) => taxonomy.modifier_class.map((entry) => entry.id),
    pick: (counts, id) => counts.modifierClass[id],
  },
  {
    title:
      "Dimension 4: binding_source — declaration-site presence signals (SFC file counts, NOT per-expression attribution)",
    ids: () => ["setup", "props", "data", "inject"],
    pick: (counts, id) => counts.bindingSignal[id],
  },
  {
    title:
      "Dimension 5: block_combination (SFCs whose top-level blocks match the combination exactly)",
    ids: (taxonomy) => taxonomy.block_combination.map((entry) => entry.id),
    pick: (counts, id) => counts.blockCombination[id],
  },
];

function scanScopeTable(hydrated) {
  return table(
    ["project", "sfc (html)", "sfc (pug)", "jsx/tsx", "html", "js"],
    [false, true, true, true, true, true],
    hydrated.map((project) => [
      `\`${project.id}\``,
      project.counts.files.sfc,
      project.counts.files.sfcPug,
      project.counts.files.jsx,
      project.counts.files.html,
      project.counts.files.js,
    ]),
  );
}

function skippedSection(vSlotOccurrences) {
  return `## Skipped (not mechanically derived by this scan)

- **binding_source per-expression attribution** — mapping each template identifier to its declaration site needs scope analysis (the croquis engine's job). The table above reports file-level declaration-site signals only (\`<script setup>\` present / \`defineProps\`-or-\`props:\` / \`data()\` / \`inject\`); the \`global\` source has no mechanical signal and is not measured at all.
- **\`v-slot\` / \`#\` shorthand** — scanned (${vSlotOccurrences} occurrences across hydrated projects) but reported nowhere above: the taxonomy has no \`v-slot\` directive row today.
- **JSX plain props** — every JSX prop is an expression binding; counting them all as \`v-bind\` would be noise, so only \`v-*\` props and \`on[A-Z]*\` event props (counted as \`v-on\`, with \`_modifier\` suffixes matched to modifier classes) are classified.
- **petite-vue built-ins** — \`v-scope\` / \`v-effect\` have no taxonomy row and land in \`custom\` (the not-in-builtin-set escape hatch).
- **Lexical limits** — pug templates are scanned line-heuristically (no pug parse); wakapi's HTML interleaves Go \`{{ }}\` template actions that the scanner skims over; TSX start tags reuse an HTML regex (single-uppercase-letter names are dropped as probable type parameters, other generics can leak); SVG/MathML descendants count via a fixed unambiguous-name set, so namespace children whose names collide with HTML tags count as \`native\`; unknown \`v-on\` modifier tokens (custom key aliases) are ignored.
- **Element kinds in scripts** — render functions and template strings inside \`.js\`/\`.ts\` sources are not scanned; only the file classes in the scan-scope table are.`;
}

/** The scope proof: a partial run must say so loudly, never read as empty. */
function scopeProof(hydratedCount, total) {
  const header = `## Scope proof (assurance rule: empty means proven-empty, never silently partial)

- **Hydrated: ${hydratedCount} of ${total} manifest projects.**`;
  if (hydratedCount < total) {
    return `${header}

> **PARTIAL CORPUS — this report measures ${hydratedCount}/${total} projects.** Every count above, including every zero, is a statement about the ${hydratedCount} hydrated projects only. The remaining ${total - hydratedCount} manifest projects are **unmeasured**, not empty. Do not read dimension coverage off this report until the full corpus is hydrated (P0-6 leaves the full-coverage step open pending corpus hydration in CI).`;
  }
  return `${header}

All manifest projects were hydrated for this run: zeros above are proven-empty over the whole registered corpus.`;
}

export function buildReport(taxonomy, projects) {
  const hydrated = projects.filter((project) => project.hydrated);
  const lines = [
    `<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/corpus-coverage.rs --write
     Verify:     rust-script tools/commands/davinci/corpus-coverage.rs --check
     Generator:  tools/davinci/corpus-coverage.mjs -->

# Corpus construct coverage

Counts of the [taxonomy.toml](./taxonomy.toml) construct dimensions observed in the **hydrated** corpus projects registered in \`tests/_fixtures/vue-ecosystem-fixtures.json\` (Davinci P0-6). This file is generated; it goes stale whenever the taxonomy, the fixtures manifest, or the set of hydrated fixture submodules changes — regenerate with \`--write\`, verify with \`--check\` (byte-compare). The \`--check\` staleness gate can only join \`tests/tooling/davinci-matrices.test.ts\` once CI hydrates the full corpus; until then the scope-proof footer below is the honesty mechanism.

## Scan scope

Sources scanned per hydrated project (from the manifest's \`vueGlobs\`, plus \`petiteVueGlobs\` for the petite-vue entries):`,
    "",
    scanScopeTable(hydrated),
    "",
    "## Per-construct counts (hydrated projects only)",
  ];
  for (const dimension of DIMENSIONS) {
    lines.push("");
    lines.push(dimensionTable(dimension.title, dimension.ids(taxonomy), hydrated, dimension.pick));
  }
  lines.push("");
  lines.push(skippedSection(hydrated.reduce((sum, project) => sum + project.counts.vSlot, 0)));
  lines.push("");
  lines.push(scopeProof(hydrated.length, projects.length));
  lines.push("");
  return lines.join("\n");
}
