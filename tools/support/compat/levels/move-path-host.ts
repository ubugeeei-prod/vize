#!/usr/bin/env node
/** Replay the whole #6834 OS path adapter move without changing its algorithms. */
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { lexicalView } from "../../levels/move-host-runtime.ts";
import { pathHostCallers, pathHostFunctions } from "./path-host-callers.ts";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const SOURCE = "davinci/vize_l0/src/path.rs";
const HOST = "crates/vize_carton/src/path.rs";
const HOST_NAMESPACE = /(?<![\p{ID_Continue}])vize_carton\s*::\s*path(?![\p{ID_Continue}])/gu;
const ORIGINAL_PATH_LAWS = "8e8124a5f6869844e332e8d0ba15cfb78a9b170e0a9b8f668a26aadc14972e0a";
const DIRECT = /(?<![\p{ID_Continue}])vize_l0(?=\s*::\s*path(?![\p{ID_Continue}]))/gu;
const GROUPS = [
  [
    "use vize_l0::{cstr, path::canonicalize_non_verbatim};",
    "use {vize_carton::path::canonicalize_non_verbatim, vize_l0::cstr};",
  ],
  [
    "use vize_l0::{String as CompactString, cstr, path::canonicalize_non_verbatim};",
    "use vize_carton::path::canonicalize_non_verbatim;\nuse vize_l0::{String as CompactString, cstr};",
  ],
] as const;
const EXPECTATION =
  '#[expect(\n    clippy::disallowed_types,\n    reason = "Preserve Windows path prefix normalization and its existing owned string"\n)]\n';

/** Change real namespace references only; retained literals/comments stay exact. */
export function rewritePathImports(text: string): string {
  for (const match of [...lexicalView(text).matchAll(DIRECT)].reverse()) {
    text = text.slice(0, match.index) + "vize_carton" + text.slice(match.index + match[0].length);
  }
  for (const [before, after] of GROUPS) {
    const view = lexicalView(text);
    const offset = view.indexOf(before);
    if (offset !== -1) {
      if (view.indexOf(before, offset + 1) !== -1) throw new Error("Duplicate path import");
      text = text.slice(0, offset) + after + text.slice(offset + before.length);
    }
  }
  return text;
}

function requireOnce(text: string, needle: string) {
  if (text.split(needle).length !== 2) throw new Error(`Missing or duplicate ${needle}`);
}

/** Validate all inputs before producing any integration writes. */
export function integrationPlan(read: (file: string) => string): Map<string, string> {
  const plan = new Map<string, string>();
  const foundation = read("davinci/vize_l0/src/lib.rs");
  const facade = read("crates/vize_carton/src/lib.rs");
  const module = read(HOST);
  const lawMarker = "#[cfg(test)]\nmod tests {";
  requireOnce(module, lawMarker);
  if (
    createHash("sha256")
      .update(module.slice(module.indexOf(lawMarker)))
      .digest("hex") !== ORIGINAL_PATH_LAWS
  )
    throw new Error("Changed original path laws");
  const declaration = "pub mod path;\n";
  const pending = foundation.includes(declaration);
  if (pending) {
    requireOnce(foundation, declaration);
    if (facade.includes(declaration)) throw new Error("Colliding path owner");
    plan.set("davinci/vize_l0/src/lib.rs", foundation.replace(declaration, ""));
    plan.set("crates/vize_carton/src/lib.rs", facade + "\n" + declaration);
    requireOnce(module, "fn strip_windows_verbatim_prefix(");
    if (module.includes(EXPECTATION)) throw new Error("Partial path integration");
    plan.set(
      HOST,
      module.replace(
        "fn strip_windows_verbatim_prefix(",
        EXPECTATION + "fn strip_windows_verbatim_prefix(",
      ),
    );
  } else {
    requireOnce(facade, declaration);
    requireOnce(module, EXPECTATION);
  }
  for (const file of Object.keys(pathHostCallers)) {
    const before = read(file);
    if (pending && [...lexicalView(before).matchAll(HOST_NAMESPACE)].length)
      throw new Error(`Partial path caller: ${file}`);
    const after = rewritePathImports(before);
    if (pending && before === after) throw new Error(`Missing original path caller: ${file}`);
    if (!pending && before !== after) throw new Error(`Unmigrated path caller: ${file}`);
    const view = lexicalView(after);
    const counts = pathHostFunctions.map(
      (name) =>
        [
          ...view.matchAll(
            new RegExp(
              `(?<![\\p{ID_Continue}])vize_carton::path::${name}(?![\\p{ID_Continue}])`,
              "gu",
            ),
          ),
        ].length,
    );
    if (
      counts.some((count, index) => count !== pathHostCallers[file][index]) ||
      [...view.matchAll(HOST_NAMESPACE)].length !== counts.reduce((a, b) => a + b, 0)
    )
      throw new Error(`Changed path host references: ${file}`);
    if (before !== after) plan.set(file, after);
  }
  return plan;
}

export function replay(phase: "moves" | "integrate" | "check", root = ROOT): void {
  const source = path.join(root, SOURCE);
  const destination = path.join(root, HOST);
  if (existsSync(source) === existsSync(destination))
    throw new Error("Missing source or path collision");
  if (phase === "moves") {
    if (existsSync(source)) execFileSync("git", ["mv", SOURCE, HOST], { cwd: root });
    return;
  }
  if (existsSync(source)) throw new Error("Commit the move-only phase before integration");
  const plan = integrationPlan((file) => readFileSync(path.join(root, file), "utf8"));
  if (phase === "check" && plan.size) throw new Error("Path ownership is not integrated");
  if (phase === "integrate") {
    for (const [file, text] of plan) writeFileSync(path.join(root, file), text);
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [phase, flag, root, ...extra] = process.argv.slice(2);
  if (
    !(phase === "moves" || phase === "integrate" || phase === "check") ||
    (flag !== undefined && (flag !== "--root" || !root)) ||
    extra.length
  )
    throw new Error("Usage: move-path-host.ts moves|integrate|check [--root PATH]");
  replay(phase, root ? path.resolve(root) : ROOT);
}
