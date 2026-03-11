import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "../../..");
const pluginDist = path.join(repoRoot, "npm/oxlint-plugin-patina/dist/index.js");
const nativeDir = path.join(repoRoot, "npm/vize-native");

function hasNativeBinary() {
  return fs.readdirSync(nativeDir).some((entry) => entry.endsWith(".node"));
}

function runPnpm(args) {
  execFileSync("pnpm", args, {
    cwd: repoRoot,
    stdio: "inherit",
  });
}

if (!hasNativeBinary()) {
  runPnpm(["-C", "npm/vize-native", "build"]);
}

if (!fs.existsSync(pluginDist)) {
  runPnpm(["-C", "npm/oxlint-plugin-patina", "build"]);
}
