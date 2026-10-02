#!/usr/bin/env node
// Replay the Vue component provider move before applying dialect scope wiring.
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { fileURLToPath } from "node:url";

const old = "davinci/vize_l1/src/markup/parse.rs";
const current = "davinci/vize_l1/src/dialect/vue3/surface.rs";

function isOwner(file: string): boolean {
  return fs.existsSync(file) && fs.readFileSync(file, "utf8").includes("pub fn parse_component");
}

function move(root: string): void {
  const source = path.join(root, old);
  const destination = path.join(root, current);
  if (fs.existsSync(destination)) {
    if (!isOwner(destination)) throw new Error(`destination is not the Vue owner: ${current}`);
    if (isOwner(source)) throw new Error(`both Vue component providers exist: ${old}, ${current}`);
    return;
  }
  if (!isOwner(source)) throw new Error(`missing Vue component provider: ${old}`);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.renameSync(source, destination);
}

function check(root: string): void {
  if (!isOwner(path.join(root, current))) throw new Error("Vue component provider is missing");
  const shim = fs.readFileSync(path.join(root, old), "utf8");
  if (!shim.includes("pub use crate::dialect::vue3::surface::{") || isOwner(path.join(root, old))) {
    throw new Error("generic markup module must retain only the compatibility export");
  }
  const dialect = fs.readFileSync(path.join(root, "davinci/vize_l1/src/dialect/vue3.rs"), "utf8");
  if (!dialect.includes("pub mod surface;"))
    throw new Error("Vue component module is not registered");
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: { root: { type: "string" } },
});
const root = values.root ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
if (positionals.length !== 1 || !["move", "check"].includes(positionals[0])) {
  throw new Error("usage: move-vue-surface-provider.ts move|check [--root PATH]");
}
(positionals[0] === "move" ? move : check)(root);
