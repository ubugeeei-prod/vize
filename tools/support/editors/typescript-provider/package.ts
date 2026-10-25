import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { assertPackageOutput, digest, stageProvider, targets } from "./artifacts.ts";
import { assertPlatformVsix } from "./assert.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const [target, artifactInput, outputInput, ...extra] = process.argv.slice(2);
assert(
  target && artifactInput && outputInput && extra.length === 0,
  "Usage: package-typescript-provider.rs <rustTarget> <artifactDir> <outVsix>",
);
assert(Object.hasOwn(targets, target), `Unknown Rust target: ${target}`);
const artifactDir = path.resolve(artifactInput),
  output = path.resolve(outputInput);
const extension = path.join(root, "editors/vscode");
assertPackageOutput(root, output);
const staging = path.join(extension, "native/typescript-projection");
assert(
  !fs.existsSync(staging),
  "Native provider staging already exists; preserve it and resolve the prior operation first",
);
const lock = path.join(root, "editors/.typescript-provider-package.lock");
const descriptor = fs.openSync(lock, "wx");
const source = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
const run = (command: string, args: string[]) =>
  execFileSync(command, args, { cwd: extension, stdio: "inherit" });
try {
  const receipt = stageProvider(root, artifactDir, staging, source, target);
  fs.mkdirSync(path.dirname(output), { recursive: true });
  run("rust-script", [
    path.join(root, "tools/commands/editors/vscode/sync-typescript-plugin.rs"),
    "stage",
  ]);
  run("rust-script", [
    path.join(root, "tools/commands/editors/vscode/run-package-bin.rs"),
    "@vscode/vsce",
    "vsce",
    "package",
    "--target",
    receipt.targetPlatform,
    "--no-dependencies",
    "--out",
    output,
  ]);
  run("rust-script", [
    path.join(root, "tools/commands/editors/vscode/sync-typescript-plugin.rs"),
    "inject",
    output,
  ]);
  assertPlatformVsix(root, output, receipt);
  fs.writeFileSync(
    `${output}.receipt.json`,
    `${JSON.stringify({ schema: "vize.platformVsix.v1", source, version: receipt.releaseVersion, target: receipt.targetPlatform, vsix: path.basename(output), bytes: fs.statSync(output).size, sha256: digest(fs.readFileSync(output)), provider: receipt }, null, 2)}\n`,
  );
} finally {
  fs.closeSync(descriptor);
  fs.unlinkSync(lock);
  fs.rmSync(staging, { recursive: true, force: true });
  try {
    fs.rmdirSync(path.dirname(staging));
  } catch {
    /* Preserve other native artifacts. */
  }
}
