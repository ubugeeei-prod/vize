#!/usr/bin/env node
// Replay the shared Vue legalizer move before applying the wiring commit.
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { fileURLToPath } from "node:url";

const files = [
  "filter.rs",
  "filter/compound.rs",
  "ids.rs",
  "on.rs",
  "slot.rs",
  "sync.rs",
  "tree.rs",
];
const moves: Array<readonly [string, string]> = [
  ["pass/legacy.rs", "dialect/legalize.rs"],
  ...files.map((file) => [`pass/legacy/${file}`, `dialect/legalize/${file}`] as const),
];

function move(root: string): void {
  const sourceRoot = path.join(root, "davinci/vize_l1_to_l2/src");
  const oldDirectory = path.join(sourceRoot, "pass/legacy");
  function existingFiles(directory: string): string[] {
    return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
      const file = path.join(directory, entry.name);
      return entry.isDirectory()
        ? existingFiles(file)
        : [path.relative(oldDirectory, file).split(path.sep).join("/")];
    });
  }
  if (fs.existsSync(oldDirectory)) {
    for (const file of existingFiles(oldDirectory)) {
      if (!files.includes(file)) throw new Error(`unreviewed legalization helper: ${file}`);
    }
  }
  const pending: Array<[string, string]> = [];
  for (const [old, current] of moves) {
    const source = path.join(sourceRoot, old);
    const destination = path.join(sourceRoot, current);
    if (fs.existsSync(destination)) {
      if (
        fs.existsSync(source) &&
        (old !== "pass/legacy.rs" || fs.readFileSync(source, "utf8").includes("pub fn run"))
      ) {
        throw new Error(`both legalization owners exist: ${old}, ${current}`);
      }
      continue;
    }
    if (!fs.existsSync(source)) throw new Error(`missing legalization owner: ${old}`);
    if (old === "pass/legacy.rs" && !fs.readFileSync(source, "utf8").includes("pub fn run")) {
      throw new Error(`source is not the legalization owner: ${old}`);
    }
    pending.push([source, destination]);
  }
  for (const [source, destination] of pending) {
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.renameSync(source, destination);
  }
  if (
    fs.existsSync(oldDirectory) &&
    fs.readdirSync(oldDirectory).every((file) => file === "filter")
  ) {
    const oldFilter = path.join(oldDirectory, "filter");
    if (fs.existsSync(oldFilter) && fs.readdirSync(oldFilter).length === 0) fs.rmdirSync(oldFilter);
    if (fs.readdirSync(oldDirectory).length === 0) fs.rmdirSync(oldDirectory);
  }
}

function check(root: string): void {
  const sourceRoot = path.join(root, "davinci/vize_l1_to_l2/src");
  for (const [old, current] of moves) {
    if (!fs.existsSync(path.join(sourceRoot, current)))
      throw new Error(`missing legalization owner: ${current}`);
    if (old !== "pass/legacy.rs" && fs.existsSync(path.join(sourceRoot, old)))
      throw new Error(`unmoved legalization helper: ${old}`);
  }
  const bridge = fs.readFileSync(path.join(sourceRoot, "pass/legacy.rs"), "utf8");
  if (!bridge.includes("pub use crate::dialect::legalize::") || bridge.includes("pub fn run")) {
    throw new Error("pass::legacy is not a narrow legalization adapter");
  }
  if (!fs.readFileSync(path.join(sourceRoot, "dialect.rs"), "utf8").includes("pub mod legalize;")) {
    throw new Error("dialect legalization is not registered");
  }
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: { root: { type: "string" } },
});
const root = values.root ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
if (positionals.length !== 1 || !["move", "check"].includes(positionals[0])) {
  throw new Error("usage: move-dialect-legalization.ts move|check [--root PATH]");
}
(positionals[0] === "move" ? move : check)(root);
