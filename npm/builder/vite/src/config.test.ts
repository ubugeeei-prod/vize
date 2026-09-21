import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  CONFIG_FILES,
  VIZE_CONFIG_JSON_SCHEMA_PATH,
  VIZE_CONFIG_PKL_SCHEMA_PATH,
  defineConfig,
} from "./config.ts";

const configSource = fs.readFileSync(
  fileURLToPath(new URL("./config.ts", import.meta.url)),
  "utf8",
);
const packageJson = JSON.parse(
  fs.readFileSync(fileURLToPath(new URL("../package.json", import.meta.url)), "utf8"),
);

assert.deepEqual(packageJson.exports?.["./internal/config-bridge"], {
  types: "./dist/internal/config-bridge.d.mts",
  import: "./dist/internal/config-bridge.mjs",
  default: "./dist/internal/config-bridge.mjs",
});

assert.doesNotMatch(
  configSource,
  /from\s+["']vize["']/,
  "vite plugin config must not statically import the vize root entrypoint",
);
assert.match(
  configSource,
  /import\(["']vize\/config["']\)/,
  "vite plugin config should lazy-load the exported vize/config subpath",
);

const bundledPath = fileURLToPath(new URL("../dist/index.mjs", import.meta.url));
if (fs.existsSync(bundledPath)) {
  // Public entries can share pack chunks. Check the modules reachable from
  // the ordinary plugin entry, without including the optional Vite+ entry.
  const visited = new Set<string>();
  function readBundle(file: string): string {
    if (visited.has(file)) return "";
    visited.add(file);
    const source = fs.readFileSync(file, "utf8");
    const dependencies = [...source.matchAll(/(?:from\s*|import\s*)["'](\.\/[^"']+\.mjs)["']/g)];
    return (
      source +
      dependencies.map((match) => readBundle(path.resolve(path.dirname(file), match[1]))).join("\n")
    );
  }
  const bundledSource = readBundle(bundledPath);
  assert.doesNotMatch(
    bundledSource,
    /(?:from\s+["']vize["']|import\(["']vize["']\))/,
    "bundled vite plugin must not import the vize root entrypoint",
  );
  assert.match(
    bundledSource,
    /import\(["']vize\/config["']\)/,
    "bundled vite plugin should keep vize/config lazy-loaded",
  );
}

const inlineConfig = {
  compiler: {
    vapor: true,
  },
};

assert.equal(defineConfig(inlineConfig), inlineConfig, "defineConfig should remain identity");
assert.ok(CONFIG_FILES.includes("vize.config.ts"), "config file names should stay exported");
assert.ok(
  VIZE_CONFIG_JSON_SCHEMA_PATH.endsWith("vize.config.schema.json"),
  "JSON schema path should resolve through the exported vize schema subpath",
);
assert.ok(
  VIZE_CONFIG_PKL_SCHEMA_PATH.endsWith("vize.pkl"),
  "Pkl schema path should resolve through the exported vize pkl subpath",
);

console.log("vite-plugin-vize config tests passed!");
