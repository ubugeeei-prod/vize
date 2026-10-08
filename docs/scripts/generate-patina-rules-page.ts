import type { RuleMetadata } from "./rules/types.ts";
import { generateRulePages } from "./rules/render.ts";
import { readdirSync, readFileSync } from "node:fs";
import { relative, resolve } from "node:path";
type SourceRule = Omit<RuleMetadata, "presets" | "category"> & { category: string | null };
type ScriptRegistry = Pick<RuleMetadata, "category" | "fixable" | "presets">;

const workspaceRoot = resolve(import.meta.dirname, "../..");
const rulesRoot = resolve(workspaceRoot, "crates/vize_patina/src/rules");
const categoryOrder =
  "Essential StronglyRecommended Recommended Accessibility HtmlConformance TypeAware Vapor Ecosystem CSS Musea Script".split(
    " ",
  );

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

const existingRulePresetsByName = readExistingRulePresets();
const sourceRulesByName = readPatinaSourceRules();
const scriptRegistryByName = readScriptRegistry();
const cssPresetRules = readOpinionatedCssRules();

const rules = [...sourceRulesByName.values()]
  .map((sourceRule) => mergeRuleMetadata(sourceRule))
  .sort((a, b) => {
    const categoryDelta = categorySortIndex(a.category) - categorySortIndex(b.category);
    return categoryDelta || a.name.localeCompare(b.name);
  });

const groupedRules = new Map<string, RuleMetadata[]>();
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

generateRulePages({ workspaceRoot, rules, groupedRules, sortedCategories, categoryLabels });

function readPatinaSourceRules() {
  const rulesByName = new Map<string, SourceRule>();
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
  const constantToName = new Map<string, string>();
  const presetConstants = new Map<string, string[]>();
  const registryByName = new Map<string, ScriptRegistry>();

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
    return new Set<string>();
  }

  return new Set([...match[1].matchAll(/"([^"]+)"/g)].map((ruleName) => ruleName[1]));
}

function mergeRuleMetadata(sourceRule: SourceRule): RuleMetadata {
  const scriptRegistry = scriptRegistryByName.get(sourceRule.name);
  const cssPresets = cssPresetRules.has(sourceRule.name) ? ["opinionated", "nuxt"] : null;
  const existingPresets = existingRulePresetsByName.get(sourceRule.name);

  return {
    ...sourceRule,
    category: scriptRegistry?.category ?? sourceRule.category ?? "Recommended",
    defaultSeverity: sourceRule.defaultSeverity ?? "warning",
    description: sourceRule.description || "",
    fixable: scriptRegistry?.fixable ?? sourceRule.fixable ?? false,
    presets: scriptRegistry?.presets ?? cssPresets ?? existingPresets ?? [],
  };
}

function readExistingRulePresets() {
  try {
    return new Map<string, string[]>(
      readFileSync(resolve(import.meta.dirname, "../content/rules/all.md"), "utf8")
        .split("\n")
        .flatMap((line): [string, string[]][] => {
          const name = line.match(/^\| (?:\[)?`([^`]+)`(?:\]\([^)]*\))? \| /)?.[1];
          const cells = line.split(" | ");
          const severity = cells.findIndex((cell) => /^`(?:error|warning)`$/.test(cell));
          return name && severity >= 0
            ? [[name, [...cells[severity + 1].matchAll(/`([^`]+)`/g)].map((preset) => preset[1])]]
            : [];
        }),
    );
  } catch {
    return new Map<string, string[]>();
  }
}

function walkRustFiles(directory: string): string[] {
  const files: string[] = [];
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

function parseQuotedField(block: string, field: string): string | null {
  const match = block.match(new RegExp(`${field}:\\s*"((?:\\\\.|[^"\\\\])*)"`));
  if (!match) return null;
  const value: unknown = JSON.parse(`"${match[1]}"`);
  if (typeof value !== "string") throw new Error(`Invalid quoted source field: ${field}`);
  return value;
}

function parseBoolField(block: string, field: string) {
  const match = block.match(new RegExp(`${field}:\\s*(true|false)`));
  return match ? match[1] === "true" : null;
}

function parseSeverity(block: string) {
  const match = block.match(/default_severity:\s*Severity::(Error|Warning)/);
  return match ? match[1].toLowerCase() : null;
}

function parseCategory(block: string) {
  const match = block.match(/category:\s*RuleCategory::([A-Za-z]+)/);
  return match ? match[1] : null;
}

function inferCategory(metaType: string) {
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

function categorySortIndex(category: string) {
  const index = categoryOrder.indexOf(category);
  return index === -1 ? categoryOrder.length : index;
}
