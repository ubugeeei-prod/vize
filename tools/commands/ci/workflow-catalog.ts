import assert from "node:assert/strict";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface WorkflowEntry {
  readonly file: string;
  readonly groupId: WorkflowGroupId;
  readonly name: string;
}

interface WorkflowGroup {
  readonly id: string;
  readonly directory: string;
  readonly title: string;
  readonly description: string;
}

const groups = [
  {
    id: "01",
    directory: "checks",
    title: "Pull request orchestration",
    description:
      "Start here for required checks, change planning, merge queues, and full qualification.",
  },
  {
    id: "02",
    directory: "release",
    title: "Release and public acceptance",
    description:
      "Publication, registry, and installed-consumer evidence. Trusted-publishing workflows are protected surfaces.",
  },
  {
    id: "03",
    directory: "performance",
    title: "Benchmarks and profiling",
    description:
      "Wall time, allocation, instruction, memory, scaling, and reproducible benchmark runs.",
  },
  {
    id: "04",
    directory: "typecheck",
    title: "Type checking and content mapping",
    description: "Typechecker, content-mapper, Corsa, and TypeScript-provider qualification.",
  },
  {
    id: "05",
    directory: "editors",
    title: "Language services and editors",
    description:
      "LSP behavior, editor packages, VSIX hosts, overlays, navigation, and hover checks.",
  },
  {
    id: "06",
    directory: "compiler",
    title: "SFC, JSX, and compiler correctness",
    description:
      "SFC compilation, JSX, bindings, templates, Vapor, recovery, and native correctness.",
  },
  {
    id: "07",
    directory: "davinci",
    title: "Davinci, Canon, and Level",
    description:
      "Davinci migration, Canon ownership, Level lowering, and cross-file graph contracts.",
  },
  {
    id: "08",
    directory: "integrations",
    title: "Builders and framework integrations",
    description: "Babel, Nuxt, Parcel, Rspack, Turbopack, Vite, Vitest, and consumer hosts.",
  },
  {
    id: "09",
    directory: "tooling",
    title: "Lint, format, and developer tooling",
    description:
      "ESLint, Oxlint, Prettier, formatter, doctor, configuration, and tooling contracts.",
  },
  {
    id: "10",
    directory: "apps",
    title: "Applications and UI packages",
    description:
      "Musea, Fresco, playground, app E2E, UI packages, and browser-facing integrations.",
  },
  {
    id: "11",
    directory: "maintenance",
    title: "Security, maintenance, and documentation",
    description:
      "Security audits, dependency upkeep, scheduled maintenance, docs, and repository hygiene.",
  },
] as const satisfies readonly WorkflowGroup[];

type WorkflowGroupId = (typeof groups)[number]["id"];

const startHere = [
  ["01", "check.yml", "Top-level required-check fan-out and final gate"],
  ["02", "pr-source-checks.yml", "Source-change planner and affected qualification"],
  ["03", "pr-rust-checks.yml", "Rust source checks and tooling shards"],
  ["04", "pr-rust-products.yml", "Affected Rust product matrix"],
  ["05", "pr-js-checks.yml", "Affected JavaScript package matrix"],
  ["06", "full-qualification.yml", "Full-suite escalation path"],
  ["07", "tool-benchmark.yml", "Published cross-tool benchmark pipeline"],
  ["08", "release.yml", "Protected release orchestration; do not casually edit"],
] as const;

export function readWorkflowEntries(root: string): readonly WorkflowEntry[] {
  const directory = path.join(root, ".github/workflows");
  const sources = path.join(root, "tools/config/pkl/workflows");
  const groupByName = new Map<string, WorkflowGroupId>();
  for (const group of groups) {
    for (const source of readdirSync(path.join(sources, group.directory))) {
      if (!source.endsWith(".pkl")) continue;
      const name = path.basename(source, ".pkl");
      assert.ok(!groupByName.has(name), `Duplicate workflow source for ${name}`);
      groupByName.set(name, group.id);
    }
  }
  return readdirSync(directory)
    .filter((file) => /\.ya?ml$/.test(file))
    .sort()
    .map((file) => {
      const source = readFileSync(path.join(directory, file), "utf8");
      const name = /^name:\s*(.+?)\s*$/m.exec(source)?.[1];
      assert.ok(name != null && name !== "", `${file} must declare a workflow name`);
      const groupId = groupByName.get(path.basename(file, path.extname(file)));
      assert.ok(groupId, `Missing Pkl workflow source for ${file}`);
      return { file, groupId, name };
    });
}

function workflowRows(entries: readonly WorkflowEntry[], groupId: WorkflowGroupId) {
  return entries
    .filter((candidate) => candidate.groupId === groupId)
    .map(
      (entry, index) =>
        [
          `${groupId}.${String(index + 1).padStart(2, "0")}`,
          entry.name,
          `[\`${entry.file}\`](./${entry.file})`,
        ] as const,
    );
}

export function githubHeadingAnchor(heading: string): string {
  return heading
    .toLowerCase()
    .replaceAll(/[^\p{L}\p{N}\s-]/gu, "")
    .replaceAll(" ", "-");
}

function groupAnchor(group: WorkflowGroup): string {
  return githubHeadingAnchor(`${group.id} ${group.title}`);
}

function markdownTable(headers: readonly string[], rows: readonly (readonly string[])[]): string[] {
  const widths = headers.map((header, column) =>
    Math.max(header.length, ...rows.map((row) => row[column]?.length ?? 0)),
  );
  const render = (row: readonly string[]) =>
    `| ${row.map((cell, column) => cell.padEnd(widths[column]!)).join(" | ")} |`;
  return [render(headers), render(widths.map((width) => "-".repeat(width))), ...rows.map(render)];
}

export function renderWorkflowCatalog(entries: readonly WorkflowEntry[]): string {
  const lines = [
    "# Workflow catalog",
    "",
    "Edit the Pkl sources linked from each group. GitHub runs the generated YAML in this directory. Each step invokes a named program. Shell recipes live under `tools/config/pkl/scripts/` and workflow drivers under `.github/scripts/workflows/`. Jobs that qualify an older checkout stage the workflow revision’s drivers before switching source.",
    "Run `node tools/commands/generate/repository-config.ts` to update the workflows and this catalog.",
    "",
    "## Start here",
    "",
    "Read the orchestration files first, then follow the reusable workflows they call.",
    "Release and trusted-publishing files are listed for navigation, but remain protected",
    "surfaces and are not routine CI-refactor targets.",
    "",
    ...markdownTable(
      ["Step", "File", "Why read it"],
      startHere.map(([step, file, purpose]) => [step, `[\`${file}\`](./${file})`, purpose]),
    ),
    "",
    "## Group index",
    "",
    ...markdownTable(
      ["Group", "Area", "Workflows"],
      groups.map((group) => [
        group.id,
        `[${group.title}](#${groupAnchor(group)})`,
        String(workflowRows(entries, group.id).length),
      ]),
    ),
    "",
  ];
  for (const group of groups) {
    const rows = workflowRows(entries, group.id);
    lines.push(
      `## ${group.id} ${group.title}`,
      "",
      group.description,
      "",
      `[Edit the Pkl sources](../../tools/config/pkl/workflows/${group.directory}/)`,
      "",
      ...markdownTable(["ID", "Workflow", "File"], rows),
    );
    lines.push("");
  }
  return `${lines.join("\n").trimEnd()}\n`;
}

const scriptPath = fileURLToPath(import.meta.url);
if (process.argv[1] != null && path.resolve(process.argv[1]) === scriptPath) {
  const root = path.resolve(path.dirname(scriptPath), "../../..");
  const directory = path.join(root, ".github/workflows");
  const output = path.join(directory, "README.md");
  const expected = renderWorkflowCatalog(readWorkflowEntries(root));
  if (process.argv.includes("--check")) {
    assert.equal(readFileSync(output, "utf8"), expected, "workflow catalog is stale");
  } else writeFileSync(output, expected);
}
