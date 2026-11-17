// `levels.inputs` (docs/davinci/decisions/2026-11-17-input-neutral-levels.md):
// input-specific code ends at L2. L2→L3, L3 and L4 name no input format
// (JSX, TSX, TSRX, SFC, Pug) in a module path, a file name or a Rust identifier,
// and they have no `lang/` module. L1 follows the target layout: one sibling per
// markup grammar and per style language, and one Vue owner.
// Present differences are listed in docs/davinci/plan/input-placement-owed.json,
// which can only shrink; the gate passes when the list is empty. Test code is exempt.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { maskRustSource } from "./davinci-storage-rust-syntax.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const owedFile = "docs/davinci/plan/input-placement-owed.json";
const neutralCrates = ["vize_l2_to_l3", "vize_l3", "vize_l4"];
const inputWords = new Set(["jsx", "tsx", "tsrx", "sfc", "pug"]);

// L1's target layout. A top-level entry is a `name.rs` file or a `name/` directory.
const l1TopLevel = new Set([
  "lib",
  "container",
  "markup",
  "style",
  "embed",
  "framework",
  "registry",
  "edit",
  "slice",
]);
const l1Children: Record<string, Set<string>> = {
  markup: new Set(["token", "profile", "grammar", "entity", "html", "pug", "tsrx"]),
  style: new Set([
    "structure",
    "classes",
    "nesting",
    "css",
    "scss",
    "sass",
    "less",
    "stylus",
    "tests",
  ]),
};

function rustFiles(root: string): string[] {
  if (!fs.existsSync(root)) return [];
  return fs.readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const file = path.join(root, entry.name);
    if (entry.isDirectory()) return rustFiles(file);
    return entry.isFile() && file.endsWith(".rs") ? [file] : [];
  });
}

function isTestFile(relative: string): boolean {
  const segments = relative.split("/");
  return (
    segments.slice(0, -1).some((segment) => /(?:^|_)tests?$/u.test(segment)) ||
    /(?:^|_)tests?\.rs$/u.test(segments.at(-1) ?? "")
  );
}

/** The lower-case words of an identifier: `JsxChildValue` → jsx, child, value. */
export function identifierWords(identifier: string): string[] {
  return identifier
    .replace(/([a-z0-9])([A-Z])/gu, "$1_$2")
    .replace(/([A-Z]+)([A-Z][a-z])/gu, "$1_$2")
    .toLowerCase()
    .split(/_+/u)
    .filter(Boolean);
}

export function namesInput(text: string): boolean {
  return (text.match(/[A-Za-z_][A-Za-z0-9_]*/gu) ?? []).some((identifier) =>
    identifierWords(identifier).some((word) => inputWords.has(word)),
  );
}

/** Non-test L2→L3/L3/L4 files that name an input in their path or code. */
function neutralOffenders(root: string): string[] {
  return neutralCrates
    .flatMap((crate) => rustFiles(path.join(root, crate, "src")))
    .map((file) => path.relative(repoRoot, file).split(path.sep).join("/"))
    .filter((relative) => !isTestFile(relative))
    .filter((relative) => {
      const inner = relative.split("/").slice(3).join("/");
      if (inner === "lang.rs" || inner.startsWith("lang/")) return true;
      if (namesInput(inner.replace(/\.rs$/u, "").replaceAll("/", " "))) return true;
      return namesInput(maskRustSource(fs.readFileSync(path.join(repoRoot, relative), "utf8")));
    });
}

function entries(directory: string): string[] {
  if (!fs.existsSync(directory)) return [];
  return [
    ...new Set(
      fs
        .readdirSync(directory, { withFileTypes: true })
        .filter((entry) => entry.isDirectory() || entry.name.endsWith(".rs"))
        .map((entry) => entry.name.replace(/\.rs$/u, "")),
    ),
  ];
}

/** L1 top-level entries, and children of `markup/` and `style/`, outside the target. */
function l1Offenders(root: string): string[] {
  const source = path.join(root, "vize_l1/src");
  const relative = (...parts: string[]) =>
    path
      .relative(repoRoot, path.join(source, ...parts))
      .split(path.sep)
      .join("/");
  const found = entries(source)
    .filter((name) => !l1TopLevel.has(name))
    .map((name) => relative(name));
  for (const [parent, allowed] of Object.entries(l1Children))
    for (const name of entries(path.join(source, parent)))
      if (!allowed.has(name)) found.push(relative(parent, name));
  return found;
}

export function offenders(): string[] {
  const root = path.join(repoRoot, "davinci");
  return [...neutralOffenders(root), ...l1Offenders(root)].sort();
}

function owed(): string[] {
  const value = JSON.parse(fs.readFileSync(path.join(repoRoot, owedFile), "utf8")) as {
    schema?: unknown;
    entries?: unknown;
  };
  assert.equal(value.schema, "vize.level-input-placement-owed");
  assert.ok(Array.isArray(value.entries));
  return value.entries as string[];
}

test("no new input-specific code outside L1 and L1→L2, and no new L1 layout drift", () => {
  const list = owed();
  assert.deepEqual(
    offenders().filter((file) => !list.includes(file)),
    [],
    "move input-specific code to its L1/L1→L2 owner (see the input-neutral levels record)",
  );
});

test("the owed list only shrinks and stays sorted", () => {
  const found = new Set(offenders());
  const list = owed();
  assert.deepEqual(
    list.filter((file) => !found.has(file)),
    [],
    "fixed entries must leave the owed list",
  );
  assert.deepEqual([...list].sort(), list, "keep the owed list sorted");
});

test("input names are matched as identifier words, not substrings", () => {
  assert.deepEqual(identifierWords("JsxChildValue"), ["jsx", "child", "value"]);
  assert.deepEqual(identifierWords("JSXElement"), ["jsx", "element"]);
  assert.deepEqual(identifierWords("_sfc_render"), ["sfc", "render"]);
  assert.equal(namesInput("enum TsrxIssue {}"), true);
  assert.equal(namesInput("fn compile_pug_block() {}"), true);
  assert.equal(namesInput("let tsxish = jsxlike + sfcs;"), false);
  assert.equal(namesInput("struct ChildValue; fn render() {}"), false);
  assert.equal(namesInput(maskRustSource('// jsx\nconst NAME: &str = "sfc";\n')), false);
});
