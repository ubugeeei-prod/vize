// The artifact's data sections: the per-rule table, the applied overrides, and
// the cross-checks that report disagreements between the engine's rule-name
// sets and the derived matrix instead of reconciling them.

import { formatTable } from "./markdown.mjs";
import { byKey } from "./ordering.mjs";
import { OVERRIDES_REL } from "./rule-parity-paths.mjs";

const croquisCell = (r) => {
  const parts = [];
  if (r.croquis.size > 0) {
    const items = [...r.croquis.keys()].sort(byKey);
    const shown = items.slice(0, 3).map((i) => `\`${i}\``);
    if (items.length > 3) shown.push(`+${items.length - 3}`);
    const sites = [...r.croquis.values()].reduce((a, b) => a + b, 0);
    parts.push(`direct ${sites}: ${shown.join(", ")}`);
  }
  if (r.ctxSites > 0) parts.push(`ctx ${r.ctxSites}`);
  return parts.length > 0 ? parts.join("; ") : "—";
};
const jsxCell = (r) => {
  if (r.jsx === "yes") return `yes (${r.jsxLane})`;
  if (r.jsxLane === "no-jsx-hooks") return "no (no JSX-reachable hooks)";
  return "no";
};
const sfcCell = (r) => (r.sfc === "yes" ? `yes (${r.sfcDetail})` : "no");

export function fullTableSection(stats) {
  const lines = [];
  lines.push("## Full table");
  lines.push("");
  lines.push("Sorted by rule name. File paths are relative to `crates/vize_patina/src/rules/`.");
  lines.push("");
  lines.push(
    formatTable(
      [
        "rule",
        "family",
        "file",
        "surfaces",
        "lint() (SFC)",
        "lint_jsx()",
        "croquis",
        "classification",
      ],
      ["left", "left", "left", "left", "left", "left", "left", "left"],
      stats.rows.map((r) => [
        `\`${r.name}\``,
        r.family,
        `\`${r.file}\``,
        r.surfaces.join(", "),
        sfcCell(r),
        jsxCell(r),
        croquisCell(r),
        `${r.classification}${r.overrideReason !== null ? " \\*" : ""}`,
      ]),
    ).trimEnd(),
  );
  lines.push("");
  return lines;
}

export function overridesSection(stats) {
  const lines = [];
  lines.push("## Overrides applied");
  lines.push("");
  if (stats.overriddenRows.length === 0) {
    lines.push(`None. Hand-corrections go in \`${OVERRIDES_REL}\`, never into this file.`);
  } else {
    lines.push(
      formatTable(
        ["rule", "classification", "reason"],
        ["left", "left", "left"],
        stats.overriddenRows.map((r) => [`\`${r.name}\``, r.classification, r.overrideReason]),
      ).trimEnd(),
    );
  }
  lines.push("");
  return lines;
}

export function crossChecksSection(matrix, stats) {
  const { model, rules } = matrix;
  const { croquisUsers, unregistered } = stats;
  const lines = [];
  lines.push("## Cross-checks");
  lines.push("");
  if (unregistered.length > 0) {
    lines.push(
      "- Rules defined but registered on no dispatch path (dead or host-only" +
        " until wired): " +
        unregistered.map((r) => `\`${r.name}\``).join(", "),
    );
  } else {
    lines.push("- Every non-musea rule is registered on at least one dispatch path.");
  }
  const engineSetGaps = [];
  for (const [setName, set] of [
    ["SEMANTIC_TEMPLATE_RULES", model.semanticTemplateRules],
    ["SHARED_SFC_DESCRIPTOR_RULES", model.sharedSfcDescriptorRules],
    ["TYPE_AWARE_RULES", model.typeAwareRules],
  ]) {
    for (const name of set) {
      const r = rules.get(name);
      if (!r) engineSetGaps.push(`\`${setName}\` names unknown rule \`${name}\``);
      else if (!r.registered) {
        engineSetGaps.push(`\`${setName}\` names \`${name}\`, which no preset registers`);
      }
    }
  }
  if (engineSetGaps.length > 0) {
    lines.push(
      "- Engine rule-name sets referencing rules outside the registered set" +
        " (gate entries that can never activate): " +
        engineSetGaps.join("; "),
    );
  } else {
    lines.push("- Every engine rule-name set entry resolves to a registered rule.");
  }
  const semanticNoCroquis = model.semanticTemplateRules.filter((name) => {
    const r = rules.get(name);
    return !r || (r.croquis.size === 0 && r.ctxSites === 0);
  });
  lines.push(
    "- `SEMANTIC_TEMPLATE_RULES` (engine-side croquis gate, `linter/engine/rule_sets.rs`)" +
      ` lists ${model.semanticTemplateRules.length} rules; ` +
      (semanticNoCroquis.length === 0
        ? "all of them show croquis usage above."
        : "these show no croquis usage above (disagreement, not reconciled): " +
          semanticNoCroquis.map((n) => `\`${n}\``).join(", ")),
  );
  const gatedNames = new Set(model.semanticTemplateRules);
  const usersOutsideGate = croquisUsers.filter(
    (r) => r.family === "template-family" && r.ctxSites > 0 && !gatedNames.has(r.name),
  );
  lines.push(
    "- Context-lane croquis users outside that gate (their template pass runs" +
      " without analysis unless another path supplies it): " +
      (usersOutsideGate.length === 0
        ? "none."
        : usersOutsideGate.map((r) => `\`${r.name}\``).join(", ")),
  );
  lines.push(
    `- Script registry: ${matrix.scriptRegistry.registered.size} dispatch entries vs` +
      ` ${matrix.scriptRegistry.allCount} names in \`ALL_BUILTIN_SCRIPT_RULE_NAMES\`` +
      (matrix.scriptRegistry.registered.size === matrix.scriptRegistry.allCount
        ? " (agree)."
        : " (**disagree** — investigate)."),
  );
  lines.push("");
  return lines;
}
