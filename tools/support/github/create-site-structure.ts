/**
 * Assembles the GitHub Pages tree from downloaded build artifacts.
 *
 * Usage: node tools/support/github/create-site-structure.ts
 *
 * Docs become the site root, the playground is served from `play/`, Musea
 * examples from `musea-examples/`, and the playground `CNAME` is copied last.
 * Each destination directory must not exist yet.
 */
import fs from "node:fs";

import {
  createDirectory,
  isEntrypoint,
  joinPath,
  runEntrypoint,
  writeFileDurably,
} from "../command-runtime.ts";

export function copyTree(source: string, destination: string): void {
  createDirectory(destination, 0o755);
  for (const entryName of fs.readdirSync(source)) {
    const sourcePath = joinPath(source, entryName);
    const destinationPath = joinPath(destination, entryName);
    if (fs.statSync(sourcePath).isDirectory()) {
      copyTree(sourcePath, destinationPath);
    } else {
      writeFileDurably(destinationPath, fs.readFileSync(sourcePath), 0o644);
    }
  }
}

export function main(): number {
  copyTree("artifacts/docs", "site");
  copyTree("artifacts/playground", "site/play");
  copyTree("artifacts/musea-examples", "site/musea-examples");
  writeFileDurably("site/CNAME", fs.readFileSync("playground/public/CNAME"), 0o644);
  return 0;
}

if (isEntrypoint(import.meta.url)) runEntrypoint(main);
