import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// Reuse the exact native TypeScript compiler already pinned for the Corsa runtime.
// This checks owned private tooling; it neither emits nor changes public packages.
const root = fileURLToPath(new URL("../../../", import.meta.url));
const require = createRequire(new URL("../../../package.json", import.meta.url));
const packagePath = require.resolve(
  `@typescript/typescript-${process.platform}-${process.arch}/package.json`,
);
const compiler = join(
  dirname(packagePath),
  "lib",
  process.platform === "win32" ? "tsc.exe" : "tsc",
);
const project = process.argv[2] ?? "tsconfig.tooling-migration.json";
const result = spawnSync(compiler, ["--noEmit", "--pretty", "false", "--project", project], {
  cwd: root,
  stdio: "inherit",
  shell: false,
});
if (result.error) throw result.error;
process.exit(result.status ?? 1);
