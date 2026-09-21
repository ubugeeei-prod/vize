import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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
      "withVue requires vite-plus installed in this project. Install your preferred version first.",
    );
  }
  const manifest = JSON.parse(readFileSync(packageFile, "utf8"));
  return { require, binary: path.resolve(path.dirname(packageFile), manifest.bin.vp) };
}

const catalogs = new Map<string, Set<string>>();

/** Only disable rules supported by the consumer's bundled Oxlint. */
export function availableVueRules(): Set<string> {
  const { binary } = resolveVitePlus();
  const cached = catalogs.get(binary);
  if (cached) return cached;
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-oxlint-rules-"));
  try {
    // A standalone project prevents recursive loading of withVue while
    // asking the installed CLI for its public rule catalog.
    writeFileSync(
      path.join(directory, "package.json"),
      '{"name":"vize-rule-catalog","private":true}',
    );
    const result = spawnSync(process.execPath, [binary, "lint", "--rules", "--format=json"], {
      cwd: directory,
      encoding: "utf8",
      timeout: 10_000,
      maxBuffer: 4 * 1024 * 1024,
    });
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
    return available;
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}
