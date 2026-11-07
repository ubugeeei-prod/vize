import assert from "node:assert/strict";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface WorkflowEntry {
  readonly file: string;
  readonly name: string;
}

const groups = [
  "Required and pull request gates",
  "Release and public acceptance",
  "Performance and profiling",
  "Editors and language services",
  "Compiler and native correctness",
  "Packages and ecosystem",
  "Maintenance and documentation",
] as const;

type WorkflowGroup = (typeof groups)[number];

export function workflowGroup(file: string): WorkflowGroup {
  if (
    /^(check|pr-|full-|e2e|title-policy|zizmor|miri|fuzz|rust-pr|queue-release-gates)/.test(file)
  ) {
    return "Required and pull request gates";
  }
  if (
    /^(release|published-|public-|pkg-pr|jsr-|corsa-package|plain-script-import-public)/.test(file)
  ) {
    return "Release and public acceptance";
  }
  if (
    /(bench|benchmark|profile|scaling|scale|memory|instruction|allocation|walltime|cold-|resource-budget|source-operation|clone-cost)/.test(
      file,
    )
  ) {
    return "Performance and profiling";
  }
  if (/^(lsp-|editor-|vscode-|typescript-|volar-|router-editor|named-sfc|css-hover)/.test(file)) {
    return "Editors and language services";
  }
  if (
    /^(canon-|cross-file|davinci-|native-|sfc-|jsx-|vapor-|binding-|component-|template-|level-|recovery-|for-bindings|defaulted-|async-)/.test(
      file,
    )
  ) {
    return "Compiler and native correctness";
  }
  if (
    /^(babel-|eslint-|prettier-|nuxt|parcel-|rspack-|turbopack-|vite|vitest-|musea-|fresco|formatter-|lint-|n8n-|doctor-|patina-|content-mapper)/.test(
      file,
    )
  ) {
    return "Packages and ecosystem";
  }
  return "Maintenance and documentation";
}

export function readWorkflowEntries(directory: string): readonly WorkflowEntry[] {
  return readdirSync(directory)
    .filter((file) => /\.ya?ml$/.test(file))
    .sort()
    .map((file) => {
      const source = readFileSync(path.join(directory, file), "utf8");
      const name = /^name:\s*(.+?)\s*$/m.exec(source)?.[1];
      assert.ok(name != null && name !== "", `${file} must declare a workflow name`);
      return { file, name };
    });
}

export function renderWorkflowCatalog(entries: readonly WorkflowEntry[]): string {
  const lines = [
    "# Workflow catalog",
    "",
    "GitHub only discovers workflow files directly under this directory, so the files",
    "cannot be organized into subdirectories. This generated catalog provides the missing",
    "navigation layer; reusable implementation belongs in `.github/actions/` or typed",
    "drivers under `tools/`.",
    "",
    "Regenerate with `node tools/commands/ci/workflow-catalog.ts`.",
    "",
  ];
  for (const group of groups) {
    lines.push(`## ${group}`, "", "| Workflow | File |", "| --- | --- |");
    for (const entry of entries.filter((candidate) => workflowGroup(candidate.file) === group)) {
      lines.push(`| ${entry.name} | [\`${entry.file}\`](./${entry.file}) |`);
    }
    lines.push("");
  }
  return `${lines.join("\n").trimEnd()}\n`;
}

const scriptPath = fileURLToPath(import.meta.url);
if (process.argv[1] != null && path.resolve(process.argv[1]) === scriptPath) {
  const root = path.resolve(path.dirname(scriptPath), "../../..");
  const directory = path.join(root, ".github/workflows");
  const output = path.join(directory, "README.md");
  const expected = renderWorkflowCatalog(readWorkflowEntries(directory));
  if (process.argv.includes("--check")) {
    assert.equal(readFileSync(output, "utf8"), expected, "workflow catalog is stale");
  } else writeFileSync(output, expected);
}
