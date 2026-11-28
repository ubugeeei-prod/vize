import { spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import {
  chmodSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

export function resolveVitePlus() {
  const require = createRequire(path.resolve("package.json"));
  let packageFile: string;
  try {
    packageFile = require.resolve("vite-plus/package.json");
  } catch {
    throw new Error(
      "defineConfig requires vite-plus installed in this project. Install your preferred version first.",
    );
  }
  const manifest = JSON.parse(readFileSync(packageFile, "utf8"));
  return { require, binary: path.resolve(path.dirname(packageFile), manifest.bin.vp) };
}

const catalogs = new Map<string, Set<string>>();
const CATALOG_CACHE_LIMIT = 8;
const CATALOG_SOURCE = "export {};\n";
const CATALOG_SOURCE_FILE = "catalog.ts";
const CATALOG_PACKAGE = '{"name":"vize-rule-catalog","private":true}';
let preparedCatalogRoot: string | undefined;

function cacheHome(): string {
  const xdg = process.env.XDG_CACHE_HOME;
  if (xdg && path.isAbsolute(xdg)) return xdg;
  if (process.platform === "win32") {
    const local = process.env.LOCALAPPDATA;
    if (local && path.isAbsolute(local)) return local;
    return path.join(os.homedir(), "AppData", "Local");
  }
  if (process.platform === "darwin") return path.join(os.homedir(), "Library", "Caches");
  return path.join(os.homedir(), ".cache");
}

function catalogRoot(): string {
  return path.join(cacheHome(), "vize", "vite-plus", "oxlint-rules-v1");
}

function catalogCacheFile(binary: string): string | undefined {
  try {
    const stat = statSync(binary);
    const key = createHash("sha256")
      .update(binary)
      .update("\0")
      .update(String(stat.mtimeMs))
      .update("\0")
      .update(String(stat.size))
      .digest("hex");
    return path.join(catalogRoot(), `${key}.json`);
  } catch {
    return undefined;
  }
}

function readCatalogCache(file: string | undefined): Set<string> | undefined {
  if (!file) return undefined;
  try {
    const metadata = lstatSync(file);
    if (!metadata.isFile() || metadata.isSymbolicLink()) return undefined;
    const parsed = JSON.parse(readFileSync(file, "utf8")) as unknown;
    if (!Array.isArray(parsed) || parsed.some((rule) => typeof rule !== "string")) return undefined;
    return new Set(parsed);
  } catch {
    return undefined;
  }
}

function writeCatalogCache(file: string | undefined, rules: Set<string>) {
  if (!file) return;
  try {
    const directory = path.dirname(file);
    if (preparedCatalogRoot !== directory) {
      ensurePrivateDirectory(directory);
      recoverInterruptedWrites(directory);
      preparedCatalogRoot = directory;
    }
    atomicWrite(file, JSON.stringify([...rules].sort()));
    pruneCatalogCache(directory, file);
  } catch {
    // A later process can spawn the catalog again.
  }
}

function ensurePrivateDirectory(directory: string): void {
  mkdirSync(directory, { recursive: true, mode: 0o700 });
  const metadata = lstatSync(directory);
  if (!metadata.isDirectory() || metadata.isSymbolicLink())
    throw new Error(`Vite+ rule catalog cache is not a private directory: ${directory}`);
  // mkdir's mode is masked and has no effect on existing directories. This is
  // our terminal namespace, so narrow it again before placing executable
  // discovery state inside it. Windows accepts chmod as a compatibility no-op.
  chmodSync(directory, 0o700);
}

function atomicWrite(file: string, source: string): void {
  const temporary = `${file}.${process.pid}.${randomUUID()}.tmp`;
  try {
    writeFileSync(temporary, source, { flag: "wx", mode: 0o600 });
    try {
      renameSync(temporary, file);
    } catch (error) {
      // Windows does not replace an existing destination. A concurrent writer
      // publishing the same immutable bytes already completed our operation.
      if (readFileSync(file, "utf8") !== source) throw error;
    }
  } finally {
    rmSync(temporary, { force: true });
  }
}

function processIsAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error instanceof Error && "code" in error && error.code === "EPERM";
  }
}

function recoverInterruptedWrites(directory: string): void {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (!entry.isFile() || !entry.name.endsWith(".tmp")) continue;
    const match = /\.([0-9]+)\.[^.]+\.tmp$/u.exec(entry.name);
    if (match && !processIsAlive(Number(match[1])))
      rmSync(path.join(directory, entry.name), { force: true });
  }
}

function pruneCatalogCache(directory: string, current: string): void {
  const entries = readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile() && /^[a-f0-9]{64}\.json$/u.test(entry.name))
    .map((entry) => {
      const file = path.join(directory, entry.name);
      return { file, modified: lstatSync(file).mtimeMs };
    })
    .sort((left, right) => right.modified - left.modified);
  const retained = new Set([current]);
  for (const entry of entries) {
    if (retained.size >= CATALOG_CACHE_LIMIT) break;
    retained.add(entry.file);
  }
  for (const entry of entries) {
    if (!retained.has(entry.file)) rmSync(entry.file, { force: true });
  }
}

function managedCatalogWorkspace(): string {
  const directory = catalogRoot();
  ensurePrivateDirectory(directory);
  recoverInterruptedWrites(directory);
  preparedCatalogRoot = directory;
  for (const [name, contents] of [
    ["package.json", CATALOG_PACKAGE],
    [CATALOG_SOURCE_FILE, CATALOG_SOURCE],
  ]) {
    const file = path.join(directory, name);
    try {
      const metadata = lstatSync(file);
      if (
        metadata.isFile() &&
        !metadata.isSymbolicLink() &&
        readFileSync(file, "utf8") === contents
      )
        continue;
    } catch {
      // Publish missing or invalid workspace inputs exclusively below.
    }
    atomicWrite(file, contents);
  }
  return directory;
}

function temporaryCatalogWorkspace(): { directory: string; cleanup(): void } {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-oxlint-rules-"));
  try {
    writeFileSync(path.join(directory, "package.json"), CATALOG_PACKAGE, {
      flag: "wx",
      mode: 0o600,
    });
    writeFileSync(path.join(directory, CATALOG_SOURCE_FILE), CATALOG_SOURCE, {
      flag: "wx",
      mode: 0o600,
    });
  } catch (error) {
    rmSync(directory, { recursive: true, force: true });
    throw error;
  }
  return {
    directory,
    cleanup: () => rmSync(directory, { recursive: true, force: true }),
  };
}

/** Vite+ can reject an ignored workspace before it forwards --rules to Oxlint. */
function hasNoLintInputs(output: string): boolean {
  try {
    const start = output.indexOf("{");
    if (start < 0) return false;
    const report: unknown = JSON.parse(output.slice(start));
    return (
      report !== null &&
      typeof report === "object" &&
      "number_of_files" in report &&
      report.number_of_files === 0 &&
      "number_of_rules" in report &&
      report.number_of_rules === null &&
      "diagnostics" in report &&
      Array.isArray(report.diagnostics) &&
      report.diagnostics.length === 0
    );
  } catch {
    return false;
  }
}

/** Only disable rules supported by the consumer's bundled Oxlint. */
export function availableVueRules(): Set<string> {
  const { binary } = resolveVitePlus();
  const cached = catalogs.get(binary);
  if (cached) return cached;
  const cacheFile = catalogCacheFile(binary);
  const stored = readCatalogCache(cacheFile);
  if (stored) {
    catalogs.set(binary, stored);
    return stored;
  }
  let directory: string;
  let managed = true;
  let cleanup = () => {};
  try {
    directory = managedCatalogWorkspace();
  } catch {
    const temporary = temporaryCatalogWorkspace();
    managed = false;
    directory = temporary.directory;
    cleanup = () => temporary.cleanup();
  }
  try {
    // A standalone project prevents recursive loading of defineConfig while
    // asking the installed CLI for its public rule catalog.
    // Vite+ may select lintable inputs before forwarding --rules. Keep that
    // input in the owned workspace: consumer dist/node_modules exclusions
    // must not discard the installed module used to discover its rules.
    const discover = (cwd: string) =>
      spawnSync(
        process.execPath,
        [
          binary,
          "lint",
          "--rules",
          "--format=json",
          "--no-ignore",
          path.join(cwd, CATALOG_SOURCE_FILE),
        ],
        { cwd, encoding: "utf8", env: process.env, timeout: 60_000, maxBuffer: 4 * 1024 * 1024 },
      );
    let result = discover(directory);
    if (managed && !result.error && result.status !== 0 && hasNoLintInputs(result.stdout)) {
      const temporary = temporaryCatalogWorkspace();
      try {
        result = discover(temporary.directory);
      } finally {
        temporary.cleanup();
      }
    }
    if (result.error && "code" in result.error && result.error.code === "ETIMEDOUT") {
      const available = new Set<string>();
      catalogs.set(binary, available);
      return available;
    }
    if (result.error || result.status !== 0) {
      throw new Error(
        `Cannot read installed Vite+ lint rules: ${result.error?.message ?? (result.stderr.trim() || result.stdout.trim() || `exit status ${result.status}`)}`,
      );
    }
    const rules = JSON.parse(result.stdout) as { scope: string; value: string }[];
    if (!Array.isArray(rules))
      throw new Error("Installed Vite+ returned an invalid lint rule catalog");
    const available = new Set(
      rules.filter((rule) => rule.scope === "vue").map((rule) => rule.value),
    );
    catalogs.set(binary, available);
    writeCatalogCache(cacheFile, available);
    return available;
  } finally {
    cleanup();
  }
}

/** @internal Reset only the process-local layer for runtime custody tests. */
export function resetAvailableVueRulesForTests(): void {
  catalogs.clear();
}
