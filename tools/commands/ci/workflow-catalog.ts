import assert from "node:assert/strict";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface WorkflowEntry {
  readonly file: string;
  readonly name: string;
}

interface WorkflowGroup {
  readonly id: string;
  readonly title: string;
  readonly description: string;
}

const groups = [
  {
    id: "01",
    title: "Pull request orchestration",
    description:
      "Start here for required checks, change planning, merge queues, and full qualification.",
  },
  {
    id: "02",
    title: "Release and public acceptance",
    description:
      "Publication, registry, and installed-consumer evidence. Trusted-publishing workflows are protected surfaces.",
  },
  {
    id: "03",
    title: "Benchmarks and profiling",
    description:
      "Wall time, allocation, instruction, memory, scaling, and reproducible benchmark runs.",
  },
  {
    id: "04",
    title: "Type checking and content mapping",
    description: "Typechecker, content-mapper, Corsa, and TypeScript-provider qualification.",
  },
  {
    id: "05",
    title: "Language services and editors",
    description:
      "LSP behavior, editor packages, VSIX hosts, overlays, navigation, and hover checks.",
  },
  {
    id: "06",
    title: "SFC, JSX, and compiler correctness",
    description:
      "SFC compilation, JSX, bindings, templates, Vapor, recovery, and native correctness.",
  },
  {
    id: "07",
    title: "Davinci, Canon, and Level",
    description:
      "Davinci migration, Canon ownership, Level lowering, and cross-file graph contracts.",
  },
  {
    id: "08",
    title: "Builders and framework integrations",
    description: "Babel, Nuxt, Parcel, Rspack, Turbopack, Vite, Vitest, and consumer hosts.",
  },
  {
    id: "09",
    title: "Lint, format, and developer tooling",
    description:
      "ESLint, Oxlint, Prettier, formatter, doctor, configuration, and tooling contracts.",
  },
  {
    id: "10",
    title: "Applications and UI packages",
    description:
      "Musea, Fresco, playground, app E2E, UI packages, and browser-facing integrations.",
  },
  {
    id: "11",
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

export function workflowGroupId(file: string): WorkflowGroupId {
  if (
    /^(check(?:-|\.)|pr-|full-|title-policy|rust-pr-target-seed|merge-queue|e2e|canonical-corpus|real-project-matrix)/.test(
      file,
    )
  ) {
    return "01";
  }
  if (/release|(^|-)published-|(^|-)public(?:-|\.yml)|^(pkg-pr|jsr-|corsa-package)/.test(file)) {
    return "02";
  }
  if (
    /(bench|benchmark|profile|scaling|scale|memory|instruction|allocation|walltime|cold-|resource-budget|source-operation|recovery-cost|clone-cost|partition-weight|persistent-batch|component-scope-binding)/.test(
      file,
    )
  ) {
    return "03";
  }
  if (
    /^(typecheck|typechecker|typescript-provider|content-mapper|corsa-|checker-|component-input-(?:diagnostic|projection))/.test(
      file,
    )
  ) {
    return "04";
  }
  if (
    /^(lsp-|editor-|vscode-|typescript-plugin|volar-|router-|named-sfc|css-hover|vue2-editor|zed-|helix-|vim-|neovim-|emacs-)/.test(
      file,
    )
  ) {
    return "05";
  }
  if (
    /^(native-|sfc-|jsx-|vapor-|binding-|template-|recovery-|for-bindings|defaulted-|async-|asset-import|ambient-template|compiler-|cross-file-script|unicode-|provide-type|expression-|css-|empty-generic|l1-|narrowed-slot|pre-textarea|scoped-nesting|source-symbolizer|typed-context|vue2-ir)/.test(
      file,
    )
  ) {
    return "06";
  }
  if (/^(canon-|cross-file|davinci-|level-|compact-collections)/.test(file)) return "07";
  if (
    /^(babel-|nuxt|parcel-|rspack-|turbopack-|vite|vitest-|plain-script-import-ref|js-install-closure|void-build|vuefes-)/.test(
      file,
    )
  ) {
    return "08";
  }
  if (
    /^(eslint-|oxlint-|prettier-|formatter-|lint-|doctor-|n8n-|patina-|workspace-|windows-config|config-|pkl-)/.test(
      file,
    )
  ) {
    return "09";
  }
  if (/^(musea-|fresco|playground-|app-|ui-|mounted-runtime)/.test(file)) return "10";
  // Group 11 is the explicit default for security, maintenance, documentation,
  // and workflows that do not belong to a more specific product area.
  return "11";
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

function workflowRows(entries: readonly WorkflowEntry[], groupId: WorkflowGroupId) {
  return entries
    .filter((candidate) => workflowGroupId(candidate.file) === groupId)
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
    "GitHub only discovers workflow files directly under this directory, so the files",
    "cannot be organized into subdirectories without changing how Actions finds them.",
    "This generated, numbered catalog is the navigation layer; reusable implementation",
    "belongs in `.github/actions/` or typed drivers under `tools/`.",
    "",
    "Regenerate with `node tools/commands/ci/workflow-catalog.ts`.",
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
    const idWidth = Math.max("ID".length, ...rows.map(([id]) => id.length));
    const workflowWidth = Math.max("Workflow".length, ...rows.map(([, name]) => name.length));
    const fileWidth = Math.max("File".length, ...rows.map(([, , file]) => file.length));
    lines.push(
      `## ${group.id} ${group.title}`,
      "",
      group.description,
      "",
      `| ${"ID".padEnd(idWidth)} | ${"Workflow".padEnd(workflowWidth)} | ${"File".padEnd(fileWidth)} |`,
      `| ${"-".repeat(idWidth)} | ${"-".repeat(workflowWidth)} | ${"-".repeat(fileWidth)} |`,
    );
    for (const [id, name, file] of rows) {
      lines.push(
        `| ${id.padEnd(idWidth)} | ${name.padEnd(workflowWidth)} | ${file.padEnd(fileWidth)} |`,
      );
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
