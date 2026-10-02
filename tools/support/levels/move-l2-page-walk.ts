#!/usr/bin/env node
// Replay the #6838 page-walk owner move before applying its wiring commit.
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { fileURLToPath } from "node:url";

const old = "davinci/vize_l1_to_l2/src/pass/walk.rs";
const current = "davinci/vize_l2/src/walk.rs";
const marker = "pub struct PageWalk";
const oldMarker = "pub(crate) struct PageWalk";

function move(root: string): void {
  const source = path.join(root, old);
  const destination = path.join(root, current);
  if (fs.existsSync(destination)) {
    if (fs.existsSync(source) && fs.readFileSync(source, "utf8").includes(oldMarker)) {
      throw new Error("both page-walk owners exist");
    }
    return;
  }
  if (!fs.existsSync(source) || !fs.readFileSync(source, "utf8").includes(oldMarker)) {
    throw new Error(`missing page-walk owner: ${old}`);
  }
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.renameSync(source, destination);
}

function check(root: string): void {
  const owner = fs.readFileSync(path.join(root, current), "utf8");
  if (!owner.includes(marker) || !owner.includes("pub fn mint_attached")) {
    throw new Error("L2 page-walk provider is missing");
  }
  if (!fs.readFileSync(path.join(root, old), "utf8").includes("pub(crate) use vize_l2::walk::")) {
    throw new Error("conversion does not use the shared L2 page walk");
  }
  if (
    !fs
      .readFileSync(path.join(root, "davinci/vize_l2/src/lib.rs"), "utf8")
      .includes("pub mod walk;")
  ) {
    throw new Error("L2 page walk is not registered");
  }
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: { root: { type: "string" } },
});
const root = values.root ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
if (positionals.length !== 1 || !["move", "check"].includes(positionals[0])) {
  throw new Error("usage: move-l2-page-walk.ts move|check [--root PATH]");
}
(positionals[0] === "move" ? move : check)(root);
