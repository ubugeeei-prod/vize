// Workspace crate discovery. Directory iteration is sorted so the generated
// artifact stays byte-stable across filesystems.

import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";

import { CRATES_DIR } from "./paths.mjs";

export function discoverCrates() {
  const crates = [];
  for (const dirent of readdirSync(CRATES_DIR, { withFileTypes: true }).sort((a, b) =>
    a.name < b.name ? -1 : a.name > b.name ? 1 : 0,
  )) {
    if (!dirent.isDirectory()) continue;
    const cargoToml = path.join(CRATES_DIR, dirent.name, "Cargo.toml");
    const srcDir = path.join(CRATES_DIR, dirent.name, "src");
    if (!existsSync(cargoToml) || !existsSync(srcDir)) continue;
    const nameMatch = /^\s*name\s*=\s*"([^"]+)"/m.exec(readFileSync(cargoToml, "utf8"));
    crates.push({
      dir: dirent.name,
      name: nameMatch ? nameMatch[1] : dirent.name,
      srcDir,
    });
  }
  return crates;
}

export function walkRustFiles(dir) {
  const files = [];
  const visit = (d) => {
    for (const dirent of readdirSync(d, { withFileTypes: true }).sort((a, b) =>
      a.name < b.name ? -1 : a.name > b.name ? 1 : 0,
    )) {
      const full = path.join(d, dirent.name);
      if (dirent.isDirectory()) visit(full);
      else if (dirent.isFile() && dirent.name.endsWith(".rs")) files.push(full);
    }
  };
  visit(dir);
  return files;
}
