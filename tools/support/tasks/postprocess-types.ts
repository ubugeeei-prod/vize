/**
 * Applies Vize-specific patches to the generated config declarations.
 *
 * Usage: node tools/support/tasks/postprocess-types.ts
 *
 * The command rewrites `npm/cli/src/types/generated.ts` below the repository
 * root. Tests can bypass the root search by setting `VIZE_REPO_ROOT` to an
 * isolated fixture workspace.
 */
import fs from "node:fs";
import path from "node:path";

import {
  isEntrypoint,
  joinPath,
  pathExists,
  printLine,
  readTextFile,
  resolvePath,
  runEntrypoint,
} from "../command-runtime.ts";

function replaceAll(content: string, search: string, replacement: string): string {
  return content.split(search).join(replacement);
}

function replaceFirst(content: string, search: string, replacement: string): string {
  const index = content.indexOf(search);
  return index < 0
    ? content
    : content.slice(0, index) + replacement + content.slice(index + search.length);
}

/**
 * Walks upward from `start` until it finds the Vize repository root.
 * Two markers avoid treating a nested fixture or package directory as the root.
 */
export function findRepoRoot(start: string): string | undefined {
  let current = resolvePath(start);
  for (;;) {
    if (
      pathExists(joinPath(current, "Cargo.toml")) &&
      pathExists(joinPath(current, "npm/cli/package.json"))
    ) {
      return current;
    }
    const parent = path.dirname(current);
    if (parent === current) return undefined;
    current = parent;
  }
}

export function generatedTypesPath(root: string): string {
  return joinPath(root, "npm/cli/src/types/generated.ts");
}

/**
 * Applies the declaration patches kept out of the generic JSON Schema step.
 * The transform is idempotent, so reruns after a partial failure converge.
 */
export function postprocessGeneratedTypes(input: string): string {
  let content = replaceAll(input, "MuseaA11YConfig", "MuseaA11yConfig");
  content = replaceAll(
    content,
    "include?: string | string[];",
    "include?: string | RegExp | (string | RegExp)[];",
  );
  content = replaceAll(
    content,
    "exclude?: string | string[];",
    "exclude?: string | RegExp | (string | RegExp)[];",
  );
  const schemaProperty =
    "  /**\n" +
    "   * JSON Schema reference for editor autocompletion\n" +
    "   */\n" +
    "  $schema?: string;\n";
  content = replaceFirst(content, schemaProperty, "");

  if (content.includes("export type LintPreset = ")) return content;

  const marker = "regenerate this file.\n */\n";
  const typeAliases =
    "\n" +
    'export type LintPreset = NonNullable<LinterConfig["preset"]>;\n' +
    "\n" +
    'export type RuleSeverity = "off" | "warn" | "error";\n' +
    "\n" +
    'export type RuleCategory = "correctness" | "suspicious" | "style" | "perf" | "a11y" | "security";\n';
  return replaceFirst(content, marker, marker + typeAliases);
}

export function main(): number {
  const configuredRoot = process.env.VIZE_REPO_ROOT;
  const repoRoot =
    configuredRoot === undefined ? findRepoRoot(process.cwd()) : resolvePath(configuredRoot);
  if (repoRoot === undefined) {
    printLine("Failed to locate the repository root for postprocess_types");
    return 1;
  }
  const outputPath = generatedTypesPath(repoRoot);
  const content = readTextFile(outputPath);
  if (content === undefined) {
    printLine(`Failed to read ${outputPath}`);
    return 1;
  }
  try {
    fs.writeFileSync(outputPath, postprocessGeneratedTypes(content));
  } catch {
    printLine(`Failed to write ${outputPath}`);
    return 1;
  }
  printLine(`Post-processed generated types: ${outputPath}`);
  return 0;
}

if (isEntrypoint(import.meta.url)) runEntrypoint(main);
