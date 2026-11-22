import { execFileSync, spawnSync } from "node:child_process";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { renderRepositoryYaml, type ConfigValue } from "../../support/ci/repository-yaml.ts";
import { readWorkflowEntries, renderWorkflowCatalog } from "../ci/workflow-catalog.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const sources = path.join(root, "tools/config/pkl");

// The product CLI already owns the repository's pinned Pkl distribution.
const cliRequire = createRequire(new URL("../../../npm/cli/package.json", import.meta.url));
const { getExePath } = cliRequire("@pkl-community/pkl") as { getExePath(): string };

type Product = { source: string; target: string; shell: boolean };

function listProducts(): Product[] {
  const products: Product[] = [];
  for (const area of ["workflows", "actions", "repository", "scripts"]) {
    const directory = path.join(sources, area);
    for (const entry of readdirSync(directory, { recursive: true, withFileTypes: true })) {
      if (!entry.isFile() || !entry.name.endsWith(".pkl")) continue;
      const source = path.join(entry.parentPath, entry.name);
      const name = path.basename(entry.name, ".pkl");
      let target = path.relative(directory, source).slice(0, -4).split(path.sep).join("/");
      if (area === "workflows") target = `.github/workflows/${name}.yml`;
      if (area === "actions") target = `.github/actions/${name}/action.yml`;
      products.push({ source, target, shell: area === "scripts" });
    }
  }
  return products.sort((left, right) => left.target.localeCompare(right.target, "en"));
}

// A single evaluator shares imported modules across the entire inventory. A raw record
// separator cannot occur inside a JSON string, including a multiline Actions command.
function evaluate(products: Product[]): ConfigValue[] {
  const separator = "\u001e";
  const result = spawnSync(
    process.platform === "win32" ? "pkl" : getExePath(),
    [
      "eval",
      "--no-project",
      "--color",
      "never",
      "--root-dir",
      sources,
      "--allowed-modules",
      "pkl:,file:,repl:text$",
      "--allowed-resources",
      "^$",
      "--module-output-separator",
      separator,
      "-x",
      '(new JsonRenderer { omitNullProperties = false }).renderDocument(new Dynamic { uri = import("pkl:reflect").Module(module).uri; value = module.output.value })',
      ...products.map(({ source }) => path.relative(sources, source)),
    ],
    { cwd: sources, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr || "Pkl evaluation failed");
  const bySource = new Map<string, ConfigValue>();
  for (const document of result.stdout.split(separator)) {
    const { uri, value } = JSON.parse(document) as { uri: string; value: ConfigValue };
    bySource.set(fileURLToPath(uri), value);
  }
  return products.map(({ source }) => {
    if (!bySource.has(source)) throw new Error(`Missing Pkl evaluation for ${source}`);
    return bySource.get(source)!;
  });
}

function ownedTarget(file: string): boolean {
  if (file === "pnpm-lock.yaml" || file.endsWith("/pnpm-lock.yaml")) return false;
  if (file.startsWith("tests/_fixtures/") || file.includes("/fixtures/")) {
    return (
      file.startsWith("tests/_fixtures/tooling/native-current-recipe-") ||
      file === "crates/vize_maestro/tests/fixtures/corsa-session-wrapper.sh"
    );
  }
  return !file.startsWith("upstream/");
}

function verifyInventory(products: Product[]): void {
  const targets = new Set(products.map(({ target }) => target));
  if (targets.size !== products.length) throw new Error("Multiple Pkl modules own the same output");
  const paths = execFileSync(
    "git",
    [
      "ls-files",
      "--cached",
      "--others",
      "--exclude-standard",
      "-z",
      "--",
      "*.yaml",
      "*.yml",
      "*.sh",
      ".envrc",
    ],
    { cwd: root, encoding: "utf8" },
  )
    .split("\0")
    .filter(Boolean);
  const missing = paths.filter((file) => ownedTarget(file) && !targets.has(file));
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

function main(): void {
  const args = process.argv.slice(2);
  if (args.length > 1 || args.some((arg) => arg !== "--check")) {
    throw new Error("Usage: node tools/commands/generate/repository-config.ts [--check]");
  }
  const check = args[0] === "--check";
  const products = listProducts();
  verifyInventory(products);
  const values = evaluate(products);
  const changes: { target: string; text: string; executable?: boolean }[] = [];
  for (const [index, { target, shell }] of products.entries()) {
    const file = path.join(root, target);
    const current = readOutput(file);
    const value = values[index]!;
    if (shell) {
      if (!isScript(value)) throw new Error(`Invalid shell script definition for ${target}`);
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
  const contents = renderWorkflowCatalog(readWorkflowEntries(directory));
  if (readOutput(catalog) !== contents) {
    if (check) throw new Error("The workflow catalog needs regeneration");
    writeFileSync(catalog, contents);
  }
  console.log(`${products.length} Pkl outputs checked, ${changes.length} updated.`);
}

main();
