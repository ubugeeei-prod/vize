import { execFileSync, spawnSync } from "node:child_process";
import {
  chmodSync,
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  automationSizeViolations,
  validateAutomationSteps,
  validateShellProgram,
} from "../../support/ci/automation-commands.ts";
import { legacyLongSteps } from "../../support/ci/automation-size-legacy.ts";
import {
  packageTaskRuntime,
  renderPackageTasks,
  validatePackageScripts,
} from "../../support/ci/package-programs.ts";
import {
  commentStyle,
  withGeneratedHeader,
  withoutGeneratedHeader,
} from "../../support/ci/generated-header.ts";
import { listPklProducts, type PklProduct } from "../../support/ci/pkl-products.ts";
import { renderRepositoryYaml, type ConfigValue } from "../../support/ci/repository-yaml.ts";
import { protectedWorkflows, workflowNameViolations } from "../../support/ci/workflow-names.ts";
import { readWorkflowEntries, renderWorkflowCatalog } from "../../ci/commands/workflow-catalog.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const sources = path.join(root, "tools/config/pkl");

// The product CLI already owns the repository's pinned Pkl distribution.
const cliRequire = createRequire(new URL("../../../npm/cli/package.json", import.meta.url));
const pkl = cliRequire("@pkl-community/pkl") as { getExePath(): string };

type Product = PklProduct;

function listProducts(): Product[] {
  return listPklProducts(root);
}

function evaluate(products: Product[]): ConfigValue[] {
  const result = spawnSync(
    process.platform === "win32" ? "pkl" : pkl.getExePath(),
    [
      "eval",
      "--no-project",
      "--color",
      "never",
      "--root-dir",
      path.join(root, "tools"),
      "--allowed-modules",
      "pkl:,file:",
      "--allowed-resources",
      "^$",
      "RepositoryProducts.pkl",
    ],
    { cwd: sources, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr || "Pkl evaluation failed");
  const records = JSON.parse(result.stdout) as Record<string, ConfigValue>;
  const values = new Map(
    Object.entries(records).map(([uri, value]) => [fileURLToPath(uri), value]),
  );
  return products.map(({ source }) => {
    if (!values.has(source)) throw new Error("Missing Pkl evaluation for " + source);
    return values.get(source)!;
  });
}

// These snapshots are compared with the original reports and workflow revisions.
const archivedFixtureInputs = new Set([
  "crates/vize_patina/tests/fixtures/issue-7905/original-command.sh",
  "tests/_fixtures/differential/lint-regressions/router-app-reachability-7932/original/command.sh",
  "tests/_fixtures/differential/lsp/string-literal-completion-original/reproduction.sh",
  "tests/_fixtures/differential/vite-native-image-identity/native-smoke-before.yml",
  "tests/_fixtures/differential/workflows/native-smoke-babel-inventory-10702/after.yml",
  "tests/_fixtures/differential/workflows/native-smoke-babel-inventory-10702/before.yml",
  "tests/tooling/fixtures/real-project-matrix-original-67e3.yml",
  "tests/tooling/fixtures/setup-wild-original-0bbbfa5d.yml",
]);

// The source-recipe observer hashes the complete step bodies from signed revisions.
const historicalStepBodies = new Set([
  "tests/_fixtures/tooling/native-current-recipe-8231.sh",
  "tests/_fixtures/tooling/native-current-recipe-8253.sh",
  "tests/_fixtures/tooling/native-current-recipe-ignore-3984.sh",
  "tests/_fixtures/tooling/native-current-recipe-integration-binary.sh",
  "tests/_fixtures/tooling/native-current-recipe-jsx-slot-8419.sh",
  "tests/_fixtures/tooling/native-current-recipe-passive-lsp-3952.sh",
  "tests/_fixtures/tooling/native-current-recipe-selected-alias-3984.sh",
]);

const vendoredDirectories = [
  "upstream/",
  "crates/vize_patina/tests/fixtures/unsafe-template-binding/reports-8752-8755/reference/",
  "tests/_fixtures/_projects/lint-dependencies/npmx.dev/",
  "tests/_fixtures/reference/vue2/",
  "tests/_fixtures/reka-tsconfig-provider/",
];

function ownedTarget(file: string): boolean {
  return (
    path.posix.basename(file) !== "pnpm-lock.yaml" &&
    !archivedFixtureInputs.has(file) &&
    !vendoredDirectories.some((directory) => file.startsWith(directory))
  );
}

function verifyInventory(products: Product[]): void {
  const targets = new Set(products.map(({ target }) => target));
  if (targets.size !== products.length) throw new Error("Multiple Pkl modules own the same output");
  for (const file of targets) {
    if (!ownedTarget(file)) throw new Error(`Pkl cannot own independently managed file: ${file}`);
  }
  const paths = execFileSync(
    "git",
    [
      "ls-files",
      "--cached",
      "--others",
      "--exclude-standard",
      "-z",
      "--",
      "package.json",
      "*/package.json",
      "*.yaml",
      "*.yml",
      "*.sh",
      "*.sh.in",
      "*.bash",
      "*.zsh",
      "*.ksh",
      "*.fish",
      "*.bat",
      "*.ps1",
      "*.cmd",
      ".envrc",
    ],
    { cwd: root, encoding: "utf8" },
  )
    .split("\0")
    .filter(Boolean);
  const missing: string[] = [];
  for (const file of paths) {
    if (!ownedTarget(file) || !lstatSync(path.join(root, file), { throwIfNoEntry: false }))
      continue;
    if (path.posix.basename(file) === "package.json") {
      const directory = path.posix.dirname(file);
      if (!targets.has(path.posix.join(directory, "scripts/package-tasks.cjs"))) {
        validatePackageScripts(root, directory, []);
      }
    } else if (!targets.has(file)) {
      missing.push(file);
    }
  }
  if (missing.length) throw new Error(`Missing Pkl sources for:\n${missing.join("\n")}`);
}

// Shared helpers that belong to no single workflow.
const workflowNeutralDirectories = new Map([
  [".github/scripts/workflows", ["shared"]],
  ["tools/config/pkl/scripts/.github/scripts/workflows", []],
  ["tools/config/pkl/parts", ["shell", "test-native-canon"]],
]);

// One stem names a workflow's Pkl source, output, driver, script and part directories,
// so a deleted or renamed workflow must not leave any of them behind.
function verifyWorkflowStems(products: Product[]): void {
  const workflows = products
    .map(({ target }) => target)
    .filter((target) => target.startsWith(".github/workflows/"))
    .map((target) => path.posix.basename(target));
  const violations = workflowNameViolations(workflows);
  const stems = new Set(workflows.map((file) => file.slice(0, -".yml".length)));
  for (const [directory, neutral] of workflowNeutralDirectories) {
    for (const entry of readdirSync(path.join(root, directory), { withFileTypes: true })) {
      if (entry.isDirectory() && !stems.has(entry.name) && !neutral.includes(entry.name))
        violations.push(`${directory}/${entry.name}: no workflow has this stem`);
    }
  }
  if (violations.length) throw new Error(`Workflow layout violations:\n${violations.join("\n")}`);
}

function readOutput(file: string): string | undefined {
  return existsSync(file) ? readFileSync(file, "utf8") : undefined;
}

function isScript(value: ConfigValue): value is { text: string; executable: boolean } {
  return (
    value !== null &&
    typeof value === "object" &&
    !Array.isArray(value) &&
    typeof value.text === "string" &&
    typeof value.executable === "boolean"
  );
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  if (args.length > 1 || args.some((arg) => arg !== "--check")) {
    throw new Error("Usage: node tools/commands/generate/repository-config.ts [--check]");
  }
  const check = args[0] === "--check";
  const products = listProducts();
  verifyInventory(products);
  verifyWorkflowStems(products);
  const values = evaluate(products);
  const runtime = await packageTaskRuntime(root);
  const changes: { target: string; text: string; executable?: boolean }[] = [];
  const sizeViolations: string[] = [];
  for (const [index, { source, target, kind }] of products.entries()) {
    const file = path.join(root, target);
    const current = readOutput(file);
    const value = values[index]!;
    const sourcePath = path.relative(root, source).split(path.sep).join("/");
    if (kind === "package") {
      const text = renderPackageTasks(root, target, value, runtime);
      if (text !== current) changes.push({ target, text });
      continue;
    }
    if (kind === "shell") {
      if (!isScript(value)) throw new Error(`Invalid shell script definition for ${target}`);
      // Historical step bodies are hashed byte for byte, so they keep their original text.
      const historical = historicalStepBodies.has(target);
      if (!historical) validateShellProgram(target, value.text);
      const text = historical
        ? value.text
        : withGeneratedHeader(value.text, commentStyle(target), sourcePath);
      if (
        current !== text ||
        (process.platform !== "win32" && ((statSync(file).mode & 0o100) !== 0) !== value.executable)
      ) {
        changes.push({ target, text, executable: value.executable });
      }
      continue;
    }
    // A tracked output can be recreated with its existing presentation after deletion.
    const tracked =
      current ??
      spawnSync("git", ["show", `:${target}`], {
        cwd: root,
        encoding: "utf8",
      }).stdout ??
      undefined;
    const template = tracked === undefined ? undefined : withoutGeneratedHeader(tracked, "hash");
    const action = target.startsWith(".github/workflows/") || target.startsWith(".github/actions/");
    if (action) {
      validateAutomationSteps(target, value);
      // Protected release files keep their reviewed commands.
      if (!(path.posix.basename(target) in protectedWorkflows))
        sizeViolations.push(
          ...automationSizeViolations(target, value).filter(
            (violation) => !legacyLongSteps.has(violation.slice(0, violation.lastIndexOf(": "))),
          ),
        );
    }
    const body = renderRepositoryYaml(value, template, !action);
    const text = withGeneratedHeader(body, "hash", sourcePath);
    if (text !== current) changes.push({ target, text });
  }
  if (sizeViolations.length)
    throw new Error(`Move long commands into named programs:\n${sizeViolations.join("\n")}`);
  if (check && changes.length) {
    throw new Error(
      `Regenerate Pkl outputs with \`node tools/commands/generate/repository-config.ts\`:\n${changes.map(({ target }) => target).join("\n")}`,
    );
  }
  for (const { target, text, executable } of changes) {
    const file = path.join(root, target);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, text);
    if (executable !== undefined) chmodSync(file, executable ? 0o755 : 0o644);
  }

  const directory = path.join(root, ".github/workflows");
  const catalog = path.join(directory, "README.md");
  const contents = withGeneratedHeader(
    renderWorkflowCatalog(readWorkflowEntries(root)),
    "html",
    "tools/config/pkl/workflows",
  );
  if (readOutput(catalog) !== contents) {
    if (check) throw new Error("The workflow catalog needs regeneration");
    writeFileSync(catalog, contents);
  }
  console.log(`${products.length} Pkl outputs checked, ${changes.length} updated.`);
}

await main();
