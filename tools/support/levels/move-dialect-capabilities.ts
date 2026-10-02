#!/usr/bin/env node
// Replay the #6841 capability-owner moves before applying their wiring commit.
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { fileURLToPath } from "node:url";

const moves = [
  ["crates/vize_armature/src/legacy.rs", "davinci/vize_l1/src/dialect/vue.rs"],
  ["davinci/vize_l1_to_l2/src/lower/caps.rs", "davinci/vize_l1/src/dialect/template.rs"],
] as const;

function move(root: string): void {
  const pending: Array<[string, string]> = [];
  for (const [old, current] of moves) {
    const source = path.join(root, old);
    const destination = path.join(root, current);
    if (fs.existsSync(destination)) {
      if (fs.existsSync(source) && fs.readFileSync(source, "utf8").includes("pub struct Legacy")) {
        throw new Error(`both capability owners exist: ${old}, ${current}`);
      }
      continue;
    }
    if (!fs.existsSync(source)) throw new Error(`missing capability owner: ${old}`);
    if (!fs.readFileSync(source, "utf8").includes("pub struct Legacy")) {
      throw new Error(`source is not the capability owner: ${old}`);
    }
    pending.push([source, destination]);
  }
  for (const [source, destination] of pending) {
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.renameSync(source, destination);
  }
}

function check(root: string): void {
  for (const [old, current] of moves) {
    if (!fs.existsSync(path.join(root, current))) {
      throw new Error(`missing moved capability owner: ${current}`);
    }
    const bridge = path.join(root, old);
    if (!fs.existsSync(bridge) || !fs.readFileSync(bridge, "utf8").includes("pub use")) {
      throw new Error(`missing capability compatibility bridge: ${old}`);
    }
    if (/pub (?:struct|enum) Legacy/u.test(fs.readFileSync(bridge, "utf8"))) {
      throw new Error(`duplicate capability owner: ${old}`);
    }
  }
  if (
    !fs
      .readFileSync(path.join(root, "davinci/vize_l1/src/lib.rs"), "utf8")
      .includes("pub mod dialect;")
  ) {
    throw new Error("L1 dialect module is not registered");
  }
  for (const version of ["vue0", "vue1", "vue2", "vue3"]) {
    if (!fs.existsSync(path.join(root, `davinci/vize_l1/src/dialect/${version}.rs`))) {
      throw new Error(`missing per-version module: ${version}`);
    }
  }
  const template = fs.readFileSync(
    path.join(root, "davinci/vize_l1/src/dialect/template.rs"),
    "utf8",
  );
  if (!template.includes("LegacyDialectCapabilities::for_dialect(version)")) {
    throw new Error("lowering projection does not use the shared capability owner");
  }
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: { root: { type: "string" } },
});
const root = values.root ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
if (positionals.length !== 1 || !["move", "check"].includes(positionals[0])) {
  throw new Error("usage: move-dialect-capabilities.ts move|check [--root PATH]");
}
(positionals[0] === "move" ? move : check)(root);
