// Artifact rendering: the file-accounting reconciliation and the summary
// bullets, assembled with the prose and data sections into the committed
// matrix. Every number here comes from ./rule-parity-summary.mjs.

import { byKey } from "./ordering.mjs";
import { derivationSection, preambleSection } from "./rule-parity-derivation.mjs";
import { CLASSIFICATIONS } from "./rule-parity-paths.mjs";
import { summarize } from "./rule-parity-summary.mjs";
import { crossChecksSection, fullTableSection, overridesSection } from "./rule-parity-tables.mjs";

function fileAccountingSection(matrix, stats) {
  const { files, ruleFiles, nonRuleFiles } = matrix;
  const { rows, moduleFiles, testFiles, helperFiles } = stats;
  const lines = [];
  lines.push("## File accounting");
  lines.push("");
  lines.push(`- \`.rs\` files under \`crates/vize_patina/src/rules/**\`: **${files.length}**`);
  lines.push(
    `- rule-defining files (exactly one \`static META\` each): **${ruleFiles.length}**` +
      ` → **${rows.length} rules**`,
  );
  lines.push(
    `- non-rule files: **${nonRuleFiles.length}** — ${moduleFiles.length} module organizers` +
      ` (a \`<name>.rs\` with a \`<name>/\` directory beside it), ${testFiles.length}` +
      ` \`*_tests.rs\` companions, ${helperFiles.length} helper/data files (rule submodules,` +
      ` shared tables, private utilities)`,
  );
  lines.push("");
  return lines;
}

function summarySection(stats) {
  const {
    rows,
    count,
    familyCounts,
    surfaceCounts,
    laneCounts,
    classCounts,
    sfcSet,
    jsxSet,
    both,
    sfcOnly,
    jsxOnly,
    neither,
    croquisUsers,
    overriddenRows,
    unregistered,
  } = stats;
  const lines = [];
  lines.push("## Summary");
  lines.push("");
  lines.push(`- **total rules: ${rows.length}**`);
  lines.push("- by family: " + [...familyCounts.entries()].map(([k, v]) => `${k} ${v}`).join(", "));
  lines.push(
    "- by surface (a rule can have several): " +
      [...surfaceCounts.entries()]
        .sort((a, b) => byKey(a[0], b[0]))
        .map(([k, v]) => `\`${k}\` ${v}`)
        .join(", "),
  );
  lines.push(
    `- path membership: SFC \`lint_sfc\` ${sfcSet.length} · JSX \`lint_jsx\` ${jsxSet.length}` +
      ` · **SFC∩JSX ${both.length}** · SFC-only ${sfcOnly.length} · JSX-only ${jsxOnly.length}` +
      ` · neither ${neither.length} (${count((r) => r.family === "musea")} musea +` +
      ` ${unregistered.length} unregistered)`,
  );
  lines.push(
    "- JSX lanes: " +
      [...laneCounts.entries()]
        .sort((a, b) => byKey(a[0], b[0]))
        .map(([k, v]) => `\`${k}\` ${v}`)
        .join(", ") +
      " — `ir` + `ir-lowered` is the markup-facade migration list" +
      ` (${(laneCounts.get("ir") ?? 0) + (laneCounts.get("ir-lowered") ?? 0)} =` +
      ` ${surfaceCounts.get("markup-facade") ?? 0} \`markup-facade\` rules)`,
  );
  lines.push(
    `- classification: ` +
      CLASSIFICATIONS.map((c) => `${c} **${classCounts.get(c)}**`).join(" · ") +
      ` (${overriddenRows.length} overridden)`,
  );
  lines.push(
    `- croquis adoption: **${croquisUsers.length}** rules touch vize_croquis` +
      ` (${count((r) => r.croquis.size > 0)} direct imports,` +
      ` ${count((r) => r.ctxSites > 0)} via context analysis)`,
  );
  lines.push("");
  return lines;
}

export function renderArtifact(matrix) {
  const stats = summarize(matrix);
  return [
    ...preambleSection(),
    ...derivationSection(matrix.model),
    ...fileAccountingSection(matrix, stats),
    ...summarySection(stats),
    ...fullTableSection(stats),
    ...overridesSection(stats),
    ...crossChecksSection(matrix, stats),
  ].join("\n");
}
