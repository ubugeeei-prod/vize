#!/usr/bin/env node
// Replay the #6841 Vue directive owner move before applying its wiring commit.
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { fileURLToPath } from "node:url";

const old = "davinci/vize_l1/src/markup/directive.rs";
const current = "davinci/vize_l1/src/dialect/vue3/directive.rs";

function move(root: string): void {
  const source = path.join(root, old);
  const destination = path.join(root, current);
  const isOwner = (file: string) =>
    fs.existsSync(file) && fs.readFileSync(file, "utf8").includes("pub struct VueDirectives;");
  if (fs.existsSync(destination)) {
    if (!isOwner(destination)) throw new Error(`destination is not the Vue owner: ${current}`);
    if (isOwner(source)) throw new Error(`both Vue directive owners exist: ${old}, ${current}`);
    return;
  }
  if (!isOwner(source)) throw new Error(`missing Vue directive owner: ${old}`);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.renameSync(source, destination);
}

function check(root: string): void {
  const provider = fs.readFileSync(path.join(root, current), "utf8");
  const core = fs.readFileSync(path.join(root, old), "utf8");
  if (!provider.includes("impl DirectiveSyntax for VueDirectives")) {
    throw new Error("Vue directive provider is missing");
  }
  if (
    !core.includes("pub use crate::dialect::vue3::VueDirectives;") ||
    /struct VueDirectives/u.test(core)
  ) {
    throw new Error("generic directive module does not retain a narrow compatibility export");
  }
  if (!core.includes("pub trait DirectiveSyntax") || !core.includes("pub struct DirectiveName")) {
    throw new Error("generic directive contracts are missing");
  }
  if (
    !fs
      .readFileSync(path.join(root, "davinci/vize_l1/src/dialect/vue3.rs"), "utf8")
      .includes("pub mod directive;")
  ) {
    throw new Error("Vue directive dialect module is not registered");
  }
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: { root: { type: "string" } },
});
const root = values.root ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
if (positionals.length !== 1 || !["move", "check"].includes(positionals[0])) {
  throw new Error("usage: move-vue-directive-syntax.ts move|check [--root PATH]");
}
(positionals[0] === "move" ? move : check)(root);
