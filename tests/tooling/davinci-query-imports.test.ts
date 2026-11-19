// `levels.query` (docs/davinci/decisions/2026-11-19-6871-query-model.md): products
// read L2 semantics through `vize_l2::query`. Patina, Maestro and Canon library code
// never names an L2 internal: `vize_l2::op`, `vize_l2::artifact`, `vize_l2::walk`, or a
// raw File record row (the `file::records` items re-exported from `vize_l2::file`).
// Present imports are listed in docs/davinci/plan/query-import-allowlist.json, one
// `<file> <family>` entry each, which can only shrink. Test code is exempt.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { maskRustSource } from "./davinci-storage-rust-syntax.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const allowlistFile = "docs/davinci/plan/query-import-allowlist.json";
const products = ["vize_patina", "vize_maestro", "vize_canon"];
const internalModules = new Set(["op", "artifact", "walk"]);

/** The item names `vize_l2::file` re-exports from its private `records` module. */
export function recordNames(fileModule: string): Set<string> {
  const block = /pub use records::\{([^}]*)\}/u.exec(maskRustSource(fileModule));
  if (!block) throw new Error("vize_l2::file re-exports no records");
  return new Set(
    block[1]
      .split(",")
      .map((name) => name.trim())
      .filter(Boolean),
  );
}

/** Every `vize_l2::…` path a source names, with use trees flattened. */
export function l2Paths(source: string): string[][] {
  const code = maskRustSource(source);
  const tokens = code.match(/[A-Za-z_]\w*|::|[{},*]/gu) ?? [];
  const found: string[][] = [];
  // tree := `*` | `{` tree, … `}` | ident [`as` ident] [`::` tree]
  const tree = (at: number, prefix: string[]): number => {
    const token = tokens[at];
    if (token === "*") {
      found.push([...prefix, "*"]);
      return at + 1;
    }
    if (token === "{") {
      let cursor = at + 1;
      while (cursor < tokens.length && tokens[cursor] !== "}") {
        cursor = tokens[cursor] === "," ? cursor + 1 : tree(cursor, prefix);
      }
      return cursor + 1;
    }
    if (token === undefined || !/^[A-Za-z_]\w*$/u.test(token)) {
      found.push(prefix);
      return at;
    }
    const segments = token === "self" ? prefix : [...prefix, token];
    let cursor = at + 1;
    if (tokens[cursor] === "as") cursor += 2;
    if (tokens[cursor] === "::") return tree(cursor + 1, segments);
    found.push(segments);
    return cursor;
  };
  for (let at = 0; at + 1 < tokens.length; at += 1) {
    if (tokens[at] === "vize_l2" && tokens[at + 1] === "::" && tokens[at - 1] !== "::") {
      tree(at + 2, []);
    }
  }
  return found;
}

/** The internal family a `vize_l2::` path reaches, if any. */
export function internalFamily(segments: string[], records: Set<string>): string | undefined {
  const [module, item] = segments;
  if (module === undefined) return undefined;
  if (internalModules.has(module)) return module;
  if (module === "file" && item !== undefined && (item === "*" || records.has(item))) {
    return "file::records";
  }
  return undefined;
}

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

export function offenders(): string[] {
  const records = recordNames(
    fs.readFileSync(path.join(repoRoot, "davinci/vize_l2/src/file.rs"), "utf8"),
  );
  const found = new Set<string>();
  for (const product of products) {
    for (const file of rustFiles(path.join(repoRoot, "crates", product, "src"))) {
      const relative = path.relative(repoRoot, file).split(path.sep).join("/");
      if (isTestFile(relative)) continue;
      for (const segments of l2Paths(fs.readFileSync(file, "utf8"))) {
        const family = internalFamily(segments, records);
        if (family !== undefined) found.add(`${relative} ${family}`);
      }
    }
  }
  return [...found].sort();
}

function allowlist(): string[] {
  const value = JSON.parse(fs.readFileSync(path.join(repoRoot, allowlistFile), "utf8")) as {
    schema?: unknown;
    entries?: unknown;
  };
  assert.equal(value.schema, "vize.query-import-allowlist");
  assert.ok(Array.isArray(value.entries));
  return value.entries as string[];
}

test("products name no new L2 internal outside vize_l2::query", () => {
  const list = allowlist();
  assert.deepEqual(
    offenders().filter((entry) => !list.includes(entry)),
    [],
    "ask through vize_l2::query::Model instead (see the query model record)",
  );
});

test("the allowlist only shrinks and stays sorted", () => {
  const found = new Set(offenders());
  const list = allowlist();
  assert.deepEqual(
    list.filter((entry) => !found.has(entry)),
    [],
    "removed imports must leave the allowlist",
  );
  assert.deepEqual([...list].sort(), list, "keep the allowlist sorted");
});

test("use trees flatten, and only internal families are reported", () => {
  const records = recordNames(
    "pub use records::{\n    Declaration, Reference, ReferenceTarget,\n};\n// pub use records::{Other};\n",
  );
  assert.deepEqual([...records], ["Declaration", "Reference", "ReferenceTarget"]);
  const paths = l2Paths(
    [
      "use vize_l2::{",
      "    file::{self, FileArtifact, ReferenceTarget as Target},",
      "    op::{Op, BindingOp},",
      "    query::Model,",
      "};",
      "use vize_l2::walk::*;",
      "fn f() { let _ = vize_l2::artifact::Artifact::new; }",
      '// vize_l2::op::Op in a comment, "vize_l2::walk" in a string',
      "#[cfg(test)] mod tests { use vize_l2::op::Op; }",
      "use other::vize_l2::op::Op;",
    ].join("\n"),
  );
  assert.deepEqual(paths, [
    ["file"],
    ["file", "FileArtifact"],
    ["file", "ReferenceTarget"],
    ["op", "Op"],
    ["op", "BindingOp"],
    ["query", "Model"],
    ["walk", "*"],
    ["artifact", "Artifact", "new"],
  ]);
  assert.deepEqual(
    paths.map((segments) => internalFamily(segments, records) ?? "-"),
    ["-", "-", "file::records", "op", "op", "-", "walk", "artifact"],
  );
});
