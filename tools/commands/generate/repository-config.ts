import {
  packageTaskRuntime,
  renderPackageTasks,
  validatePackageScripts,
} from "../../support/ci/package-programs.ts";
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
  validateAutomationSteps,
  validateShellProgram,
} from "../../support/ci/automation-commands.ts";
import { renderRepositoryYaml, type ConfigValue } from "../../support/ci/repository-yaml.ts";
import { readWorkflowEntries, renderWorkflowCatalog } from "../ci/workflow-catalog.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const sources = path.join(root, "tools/config/pkl");

// The product CLI already owns the repository's pinned Pkl distribution.
const cliRequire = createRequire(new URL("../../../npm/cli/package.json", import.meta.url));
const pkl = cliRequire("@pkl-community/pkl") as { getExePath(): string };

type Product = { source: string; target: string; kind: "yaml" | "shell" | "package" };

function listProducts(): Product[] {
  const products: Product[] = [];
  for (const area of ["workflows", "actions", "repository", "scripts", "package-tasks"]) {
    const directory = path.join(sources, area);
    for (const entry of readdirSync(directory, { recursive: true, withFileTypes: true })) {
      if (!entry.isFile() || !entry.name.endsWith(".pkl")) continue;
      const source = path.join(entry.parentPath, entry.name);
      const name = path.basename(entry.name, ".pkl");
      let target = path.relative(directory, source).slice(0, -4).split(path.sep).join("/");
      if (area === "workflows") target = `.github/workflows/${name}.yml`;
      if (area === "actions") target = `.github/actions/${target}/action.yml`;
      if (area === "package-tasks")
        target = path.posix.join(path.posix.dirname(target), "scripts/package-tasks.cjs");
      products.push({
        source,
        target,
        kind: area === "scripts" ? "shell" : area === "package-tasks" ? "package" : "yaml",
      });
    }
  }
  return products.sort((left, right) => left.target.localeCompare(right.target, "en"));
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
      sources,
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

function ownedTarget(file: string): boolean {
  if (file === "pnpm-lock.yaml" || file.endsWith("/pnpm-lock.yaml")) return false;
  if (file.startsWith("tests/_fixtures/") || file.includes("/fixtures/")) {
    return (
      file.startsWith("tests/_fixtures/tooling/native-current-recipe-") ||
      file.startsWith("tools/commands/fixtures/project_deadline/programs/") ||
      file === "crates/vize_maestro/tests/fixtures/corsa-session-wrapper.sh"
    );
  }
  return !file.startsWith("upstream/");
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
  const values = evaluate(products);
  const runtime = await packageTaskRuntime(root);
  const changes: { target: string; text: string; executable?: boolean }[] = [];
  for (const [index, { target, kind }] of products.entries()) {
    const file = path.join(root, target);
    const current = readOutput(file);
    const value = values[index]!;
    if (kind === "package") {
      const text = renderPackageTasks(root, target, value, runtime);
      if (text !== current) changes.push({ target, text });
      continue;
    }
    if (kind === "shell") {
      if (!isScript(value)) throw new Error(`Invalid shell script definition for ${target}`);
      if (!target.startsWith("tests/_fixtures/")) validateShellProgram(target, value.text);
      if (
        current !== value.text ||
        (process.platform !== "win32" && ((statSync(file).mode & 0o100) !== 0) !== value.executable)
      ) {
        changes.push({ target, text: value.text, executable: value.executable });
      }
      continue;
    }
    // A tracked output can be recreated with its existing presentation after deletion.
    const template =
      current ??
      spawnSync("git", ["show", `:${target}`], {
        cwd: root,
        encoding: "utf8",
      }).stdout ??
      undefined;
    const action = target.startsWith(".github/workflows/") || target.startsWith(".github/actions/");
    if (action) validateAutomationSteps(target, value);
    const text = renderRepositoryYaml(value, template, !action);
    if (text !== current) changes.push({ target, text });
  }
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
  const contents = renderWorkflowCatalog(readWorkflowEntries(root));
  if (readOutput(catalog) !== contents) {
    if (check) throw new Error("The workflow catalog needs regeneration");
    writeFileSync(catalog, contents);
  }
  console.log(`${products.length} Pkl outputs checked, ${changes.length} updated.`);
}

await main();
