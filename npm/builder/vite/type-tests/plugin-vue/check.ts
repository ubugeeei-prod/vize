import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const fixture = path.dirname(fileURLToPath(import.meta.url));
const vitePlugin = path.resolve(fixture, "../..");

function run(command, args, cwd) {
  execFileSync(command, args, { cwd, stdio: "inherit" });
}

run("vp", ["pack"], vitePlugin);
run("vp", ["exec", "npm", "ci", "--ignore-scripts", "--no-audit", "--no-fund"], fixture);

// Type-check the declaration we just built against a consumer's real Vite 8
// installation. A workspace symlink would resolve vite-plus from the producer.
const installed = path.join(fixture, "node_modules/@vizejs/vite-plugin");
const installedDist = path.join(installed, "dist");
rmSync(installed, { recursive: true, force: true });
mkdirSync(installedDist, { recursive: true });
for (const file of readdirSync(path.join(vitePlugin, "dist"))) {
  if (file.endsWith(".d.mts")) {
    cpSync(path.join(vitePlugin, "dist", file), path.join(installedDist, file));
  }
}
writeFileSync(
  path.join(installed, "package.json"),
  JSON.stringify({
    name: "@vizejs/vite-plugin",
    type: "module",
    exports: { "./vite-plus": { types: "./dist/vite-plus.d.mts" } },
  }),
);
run(
  process.execPath,
  [path.join(fixture, "node_modules/typescript/bin/tsc"), "-p", fixture],
  fixture,
);
