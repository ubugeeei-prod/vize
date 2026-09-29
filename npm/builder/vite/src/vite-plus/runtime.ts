import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
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
    return path.join(os.tmpdir(), "vize-oxlint-rules", `${key}.json`);
  } catch {
    return undefined;
  }
}

function readCatalogCache(file: string | undefined): Set<string> | undefined {
  if (!file) return undefined;
  try {
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
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, JSON.stringify([...rules].sort()), { mode: 0o600 });
  } catch {
    // A later process can spawn the catalog again.
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
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-oxlint-rules-"));
  try {
    // A standalone project prevents recursive loading of defineConfig while
    // asking the installed CLI for its public rule catalog.
    writeFileSync(
      path.join(directory, "package.json"),
      '{"name":"vize-rule-catalog","private":true}',
    );
    const result = spawnSync(process.execPath, [binary, "lint", "--rules", "--format=json"], {
      cwd: directory,
      encoding: "utf8",
      timeout: 60_000,
      maxBuffer: 4 * 1024 * 1024,
    });
    if (result.error && "code" in result.error && result.error.code === "ETIMEDOUT") {
      const available = new Set<string>();
      catalogs.set(binary, available);
      return available;
    }
    if (result.error || result.status !== 0) {
      throw new Error(
        `Cannot read installed Vite+ lint rules: ${result.error?.message ?? result.stderr}`,
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
    rmSync(directory, { recursive: true, force: true });
  }
}
