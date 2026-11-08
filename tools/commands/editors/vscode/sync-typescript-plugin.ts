import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { isEntrypoint, runEntrypoint } from "../../../support/command-runtime.ts";

const usage =
  "Usage: node tools/commands/editors/vscode/sync-typescript-plugin.ts <stage|inject> [vsix]";
const packagePath = "node_modules/@vizejs/typescript-vue-plugin";
export const pluginFiles = [
  "README.md",
  "component-contracts.cjs",
  "import-resolution.cjs",
  "index.cjs",
  "index.d.cts",
  "language-service.cjs",
  "module-resolution.cjs",
  "package.json",
  "project-options.cjs",
  "provider.cjs",
  "source-documents.cjs",
  "virtual-modules.cjs",
] as const;

function repositoryRoot(): string {
  const configured = process.env.VIZE_REPO_ROOT?.trim();
  if (configured) return path.resolve(configured);
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
}

function uniqueSibling(parent: string, prefix: string): string {
  return path.join(
    parent,
    `${prefix}-${process.pid}-${Date.now()}-${Math.random().toString(16).slice(2)}`,
  );
}

function validatePluginFiles(source: string): void {
  if (!fs.statSync(source, { throwIfNoEntry: false })?.isDirectory()) {
    throw new Error(`TypeScript Vue plugin source directory does not exist: ${source}`);
  }
  for (const file of pluginFiles) {
    const filename = path.join(source, file);
    if (!fs.statSync(filename, { throwIfNoEntry: false })?.isFile()) {
      throw new Error(`TypeScript Vue plugin source file is missing: ${filename}`);
    }
  }
}

function copyPluginFiles(source: string, target: string): void {
  for (const file of pluginFiles) fs.copyFileSync(path.join(source, file), path.join(target, file));
}

function replaceStagedPlugin(staged: string, target: string, backup: string): void {
  if (fs.existsSync(target)) fs.renameSync(target, backup);
  try {
    fs.renameSync(staged, target);
  } catch (error) {
    if (fs.existsSync(backup)) fs.renameSync(backup, target);
    throw error;
  }
  if (fs.existsSync(backup)) fs.rmSync(backup, { recursive: true });
}

export function stagePlugin(source: string, target: string): void {
  validatePluginFiles(source);
  const parent = path.dirname(target);
  fs.mkdirSync(parent, { recursive: true });
  const staged = uniqueSibling(parent, ".typescript-vue-plugin-stage");
  const backup = uniqueSibling(parent, ".typescript-vue-plugin-backup");
  fs.mkdirSync(staged);
  try {
    copyPluginFiles(source, staged);
    replaceStagedPlugin(staged, target, backup);
  } catch (error) {
    fs.rmSync(staged, { force: true, recursive: true });
    throw error;
  }
}

export function injectPlugin(source: string, vsixInput: string): void {
  const vsix = path.resolve(vsixInput);
  if (!fs.existsSync(vsix)) throw new Error(`VSIX does not exist: ${vsix}`);
  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vsix-plugin-"));
  try {
    stagePlugin(source, path.join(temporaryRoot, "extension", packagePath));
    const entries = pluginFiles.map((file) => `extension/${packagePath}/${file}`);
    const result = spawnSync("zip", ["-X", "-q", vsix, ...entries], {
      cwd: temporaryRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    if (result.error !== undefined) throw new Error(`failed to run zip: ${result.error.message}`);
    if (result.status !== 0) {
      throw new Error(
        `zip failed with status ${result.status ?? 1}\n${result.stderr ?? ""}${result.stdout ?? ""}`,
      );
    }
  } finally {
    fs.rmSync(temporaryRoot, { force: true, recursive: true });
  }
}

function main(args = process.argv.slice(2)): void {
  const root = repositoryRoot();
  const source = path.join(root, "npm/typescript-plugin");
  const extension = path.join(root, "editors/vscode");
  const [command, vsix] = args;
  if (command === "stage") {
    stagePlugin(source, path.join(extension, packagePath));
  } else if (command === "inject") {
    injectPlugin(source, vsix ?? path.join(process.cwd(), "dist/vize.vsix"));
  } else if (command === "--help" || command === "-h") {
    process.stdout.write(`${usage}\n`);
  } else if (command === undefined) {
    throw new Error(usage);
  } else {
    throw new Error(`unknown command ${command}\n\n${usage}`);
  }
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main());
