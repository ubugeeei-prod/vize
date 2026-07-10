import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { relative, resolve } from "node:path";

const require = createRequire(import.meta.url);

const workspaceRoot = resolve(import.meta.dirname, "../..");
const rulesRoot = resolve(workspaceRoot, "crates/vize_patina/src/rules");
const githubBlobBase = "https://github.com/ubugeeei-prod/vize/blob/main/";

const categoryOrder = [
  "Essential",
  "StronglyRecommended",
  "Recommended",
  "Accessibility",
  "HtmlConformance",
  "TypeAware",
  "Vapor",
  "Ecosystem",
  "CSS",
  "Musea",
  "Script",
];

const categoryLabels = {
  Accessibility: "Accessibility",
  CSS: "CSS",
  Ecosystem: "Ecosystem",
  Essential: "Essential",
  HtmlConformance: "HTML Conformance",
  Musea: "Musea",
  Recommended: "Recommended",
  Script: "Script",
  StronglyRecommended: "Strongly Recommended",
  TypeAware: "Type Aware",
  Vapor: "Vapor",
};

const presetLabels = {
  "general-recommended": "happy-path",
};

const nativeRulesByName = loadNativeRulesByName();
const sourceRulesByName = readPatinaSourceRules();
const scriptRegistryByName = readScriptRegistry();
const cssPresetRules = readOpinionatedCssRules();

const rules = [...sourceRulesByName.values()]
  .map((sourceRule) => mergeRuleMetadata(sourceRule))
  .sort((a, b) => {
    const categoryDelta = categorySortIndex(a.category) - categorySortIndex(b.category);
    return categoryDelta || a.name.localeCompare(b.name);
  });

const groupedRules = new Map();
for (const rule of rules) {
  const group = groupedRules.get(rule.category) ?? [];
  group.push(rule);
  groupedRules.set(rule.category, group);
}

const sortedCategories = [
  ...categoryOrder.filter((category) => groupedRules.has(category)),
  ...[...groupedRules.keys()]
    .filter((category) => !categoryOrder.includes(category))
    .sort((a, b) => a.localeCompare(b)),
];

const lines = [
  "---",
  "title: All Patina Rules",
  "---",
  "",
  "# All Patina Rules",
  "",
  `This page lists all ${rules.length} Patina rule implementations declared under \`crates/vize_patina/src/rules\`. The category pages keep the longer examples; this page is the compact reference for coverage, default severity, preset membership, fixability, and source implementation.`,
  "",
  "Preset names use Vize CLI terminology. The oxlint plugin metadata name `general-recommended` is shown here as `happy-path`. `_none_` means the rule is opt-in, host-driven, or outside the bundled lint presets.",
  "",
  "## Categories",
  "",
  "| Category | Rules |",
  "| --- | ---: |",
];

for (const category of sortedCategories) {
  const label = categoryLabels[category] ?? category;
  const count = groupedRules.get(category).length;
  lines.push(`| [${label}](#${slugify(`${label} ${count}`)}) | ${count} |`);
}

for (const category of sortedCategories) {
  const group = groupedRules.get(category);
  const label = categoryLabels[category] ?? category;
  lines.push(
    "",
    `## ${label} (${group.length})`,
    "",
    "| Rule | Severity | Presets | Fixable | Implementation | Description |",
    "| --- | --- | --- | --- | --- | --- |",
  );

  for (const rule of group) {
    lines.push(
      [
        code(rule.name),
        code(rule.defaultSeverity),
        formatPresets(rule.presets),
        rule.fixable ? "Yes" : "No",
        implementationLink(rule),
        escapeCell(rule.description),
      ]
        .join(" | ")
        .replace(/^/, "| ")
        .replace(/$/, " |"),
    );
  }
}

lines.push("");

writeFileSync(resolve(import.meta.dirname, "../content/rules/all.md"), lines.join("\n"));

function loadNativeRulesByName() {
  try {
    const { getPatinaRules } = require("../../npm/native");
    return new Map(getPatinaRules().map((rule) => [rule.name, rule]));
  } catch {
    return new Map();
  }
}

function readPatinaSourceRules() {
  const rulesByName = new Map();
  const metaPattern =
    /static\s+[A-Z_]*META[A-Z_]*\s*:\s*(RuleMeta|ScriptRuleMeta|CssRuleMeta|MuseaRuleMeta)\s*=\s*\w+\s*\{([\s\S]*?)\n\};/g;

  for (const filePath of walkRustFiles(rulesRoot)) {
    const source = readFileSync(filePath, "utf8");
    let match;
    while ((match = metaPattern.exec(source))) {
      const [, metaType, block] = match;
      const name = parseQuotedField(block, "name");
      if (!name) {
        continue;
      }

      const relativePath = relative(workspaceRoot, filePath);
      const line = source.slice(0, match.index).split("\n").length;
      rulesByName.set(name, {
        name,
        category: parseCategory(block) ?? inferCategory(metaType),
        defaultSeverity: parseSeverity(block) ?? "warning",
        description: parseQuotedField(block, "description") ?? "",
        fixable: parseBoolField(block, "fixable") ?? false,
        implementationLine: line,
        implementationPath: relativePath,
        metaType,
      });
    }
  }

  return rulesByName;
}

function readScriptRegistry() {
  const namesSource = readFileSync(
    resolve(workspaceRoot, "crates/vize_patina/src/linter/script_rules/registry/names.rs"),
    "utf8",
  );
  const registrySource = readFileSync(
    resolve(workspaceRoot, "crates/vize_patina/src/linter/script_rules/registry.rs"),
    "utf8",
  );
  const rulesSource = readFileSync(
    resolve(workspaceRoot, "crates/vize_patina/src/linter/script_rules/registry/rules.rs"),
    "utf8",
  );
  const presetSource = `${registrySource}\n${rulesSource}`;
  const constantToName = new Map();
  const presetConstants = new Map();
  const registryByName = new Map();

  for (const match of namesSource.matchAll(/const\s+(RULE_[A-Z0-9_]+):\s*&str\s*=\s*"([^"]+)"/g)) {
    constantToName.set(match[1], match[2]);
  }

  for (const match of presetSource.matchAll(
    /const\s+([A-Z_]+_PRESETS):\s*&\[&str\]\s*=\s*&\[([^\]]*)\]/g,
  )) {
    presetConstants.set(
      match[1],
      [...match[2].matchAll(/"([^"]+)"/g)].map((preset) => preset[1]),
    );
  }

  const entryPattern =
    /BuiltinScriptRuleEntry\s*\{[^}]*rule_name:\s*(RULE_[A-Z0-9_]+),[^}]*category:\s*"([^"]+)",[^}]*fixable:\s*(true|false),[^}]*presets:\s*([A-Z_]+_PRESETS),/g;
  for (const match of rulesSource.matchAll(entryPattern)) {
    const [, ruleConstant, category, fixable, presetsConstant] = match;
    const name = constantToName.get(ruleConstant);
    if (!name) {
      continue;
    }
    registryByName.set(name, {
      category,
      fixable: fixable === "true",
      presets: presetConstants.get(presetsConstant) ?? [],
    });
  }

  return registryByName;
}

function readOpinionatedCssRules() {
  const presetSource = readFileSync(
    resolve(workspaceRoot, "crates/vize_patina/src/preset.rs"),
    "utf8",
  );
  const match = presetSource.match(
    /const\s+OPINIONATED_CSS_RULE_NAMES:\s*&\[&str\]\s*=\s*&\[([\s\S]*?)\];/,
  );
  if (!match) {
    return new Set();
  }

  return new Set([...match[1].matchAll(/"([^"]+)"/g)].map((ruleName) => ruleName[1]));
}

function mergeRuleMetadata(sourceRule) {
  const nativeRule = nativeRulesByName.get(sourceRule.name);
  const scriptRegistry = scriptRegistryByName.get(sourceRule.name);
  const cssPresets = cssPresetRules.has(sourceRule.name) ? ["opinionated", "nuxt"] : null;

  return {
    ...sourceRule,
    category:
      scriptRegistry?.category ?? sourceRule.category ?? nativeRule?.category ?? "Recommended",
    defaultSeverity: sourceRule.defaultSeverity ?? nativeRule?.defaultSeverity ?? "warning",
    description: sourceRule.description || nativeRule?.description || "",
    fixable: scriptRegistry?.fixable ?? sourceRule.fixable ?? nativeRule?.fixable ?? false,
    presets: scriptRegistry?.presets ?? cssPresets ?? nativeRule?.presets ?? [],
  };
}

function walkRustFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const fullPath = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      if (entry.name !== "snapshots") {
        files.push(...walkRustFiles(fullPath));
      }
      continue;
    }
    if (entry.isFile() && entry.name.endsWith(".rs")) {
      files.push(fullPath);
    }
  }
  return files;
}

function parseQuotedField(block, field) {
  const match = block.match(new RegExp(`${field}:\\s*"((?:\\\\.|[^"\\\\])*)"`));
  return match ? JSON.parse(`"${match[1]}"`) : null;
}

function parseBoolField(block, field) {
  const match = block.match(new RegExp(`${field}:\\s*(true|false)`));
  return match ? match[1] === "true" : null;
}

function parseSeverity(block) {
  const match = block.match(/default_severity:\s*Severity::(Error|Warning)/);
  return match ? match[1].toLowerCase() : null;
}

function parseCategory(block) {
  const match = block.match(/category:\s*RuleCategory::([A-Za-z]+)/);
  return match ? match[1] : null;
}

function inferCategory(metaType) {
  if (metaType === "CssRuleMeta") {
    return "CSS";
  }
  if (metaType === "MuseaRuleMeta") {
    return "Musea";
  }
  if (metaType === "ScriptRuleMeta") {
    return "Script";
  }
  return null;
}

function categorySortIndex(category) {
  const index = categoryOrder.indexOf(category);
  return index === -1 ? categoryOrder.length : index;
}

function code(value) {
  return `\`${escapeCell(value)}\``;
}

function escapeCell(value) {
  return String(value)
    .replaceAll("|", "\\|")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replace(/\s+/g, " ")
    .trim();
}

function formatPresets(presets) {
  if (presets.length === 0) {
    return "_none_";
  }

  return presets
    .map((preset) => presetLabels[preset] ?? preset)
    .map(code)
    .join(", ");
}

function implementationLink(rule) {
  const url = `${githubBlobBase}${rule.implementationPath}#L${rule.implementationLine}`;
  return `[source](${url})`;
}

function slugify(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}
