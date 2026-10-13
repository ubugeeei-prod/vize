/**
 * Copies every built `.node` file below a package into an artifact directory.
 *
 * Usage: node tools/support/github/collect-native-artifacts.ts
 *   <source-dir> <output-dir> <label>
 *
 * `node_modules` directories are skipped, files are flattened to their base
 * names, and the copied paths are listed in shortlex order.
 */
import fs from "node:fs";
import path from "node:path";

import {
  commandArguments,
  compareShortlex,
  createDirectory,
  isDirectory,
  isEntrypoint,
  isRegularFile,
  joinPath,
  printLine,
  readDirectoryNames,
  runEntrypoint,
  writeFileDurably,
} from "../command-runtime.ts";

const usage =
  "Usage: node tools/support/github/collect-native-artifacts.ts <source-dir> <output-dir> <label>";

export function collectNodeFiles(directory: string, collected: string[] = []): string[] {
  for (const entryName of readDirectoryNames(directory)) {
    if (entryName === "node_modules") continue;
    const entryPath = joinPath(directory, entryName);
    if (isDirectory(entryPath)) {
      collectNodeFiles(entryPath, collected);
    } else if (entryName.endsWith(".node") && isRegularFile(entryPath)) {
      collected.push(entryPath);
    }
  }
  return collected;
}

/**
 * Ensures the destination directory exists before artifacts are copied.
 * Release workflows sometimes download artifacts into package directories that
 * already exist, so an existing directory keeps reruns idempotent.
 */
function ensureDirectory(target: string): void {
  if (!isDirectory(target)) createDirectory(target, 0o755);
}

export function main(args: readonly string[]): number {
  if (args.length !== 3) {
    printLine(usage);
    return 1;
  }
  const [sourceDir, outputDir, label] = args as [string, string, string];
  const collected = collectNodeFiles(sourceDir);
  if (collected.length === 0) {
    printLine(`No ${label} .node files found after build.`);
    return 1;
  }
  ensureDirectory(outputDir);
  const copied: string[] = [];
  for (const file of collected) {
    const destination = joinPath(outputDir, path.basename(file));
    writeFileDurably(destination, fs.readFileSync(file), 0o644);
    copied.push(destination);
  }
  copied.sort(compareShortlex);
  printLine(`Collected ${label} artifacts:`);
  for (const file of copied) printLine(`  ${file}`);
  return 0;
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
