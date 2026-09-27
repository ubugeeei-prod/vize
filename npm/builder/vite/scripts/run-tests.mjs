import { spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { nativePreparationIsActive } from "../../../native/scripts/test-preparation.mjs";

const packageDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(import.meta.url);

export function requirePreparedNative(nativeDir, loadBinding = require) {
  const bindings = readdirSync(nativeDir).filter(
    (name) => name.startsWith("vize-vitrine.") && name.endsWith(".node"),
  );
  if (bindings.length !== 1) {
    throw new Error(
      `Prepared Vite tests require exactly one local native binding, found ${bindings.length}. Run vp run --workspace-root build:native:test first.`,
    );
  }
  // Load the local addon directly so a missing or incompatible build cannot
  // silently fall back to an installed platform package.
  loadBinding(join(nativeDir, bindings[0]));
}

export function runViteTests(prepared, run = spawnSync) {
  const options = {
    cwd: packageDir,
    env: process.env,
    stdio: "inherit",
    shell: process.platform === "win32",
  };
  if (!prepared) {
    const build = run("pnpm", ["--dir", "../../native", "build:debug"], options);
    if (build.status !== 0) return build.status ?? 1;
  }
  return run("pnpm", ["run", "test:prepared"], options).status ?? 1;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  if (args.length === 1 && args[0] === "--require-prepared") {
    requirePreparedNative(resolve(packageDir, "../../native"));
  } else if (args.length === 0) {
    const prepared = nativePreparationIsActive(resolve(packageDir, "../../native"));
    if (prepared) console.log("Vite tests reuse the active root native preparation.");
    process.exitCode = runViteTests(prepared);
  } else {
    throw new Error("usage: run-tests.mjs [--require-prepared]");
  }
}
