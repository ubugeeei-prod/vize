import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const libraryRoots = [
  "crates/vize_davinci/src",
  "crates/vize_sinopia/src",
  "crates/vize_disegno/src",
  "crates/vize_ricalco/src",
];
const davinciOptRoot = "crates/vize_davinci/src/bin/davinci-opt/";

type VecCategory = "contract" | "analysis" | "lower" | "pass" | "emit";
type VecPolicy = { category: VecCategory; max: number };

const categoryReasons: Record<VecCategory, string> = {
  contract: "variable-length owned Folio/S2 contract data",
  analysis: "unbounded diagnostics, lookup storage, and traversal results",
  lower: "source-sized lowering worklists and owned results",
  pass: "source-sized pass facts, provenance, and traversal worklists",
  emit: "ordered emitter buffers whose size follows the document",
};

function policy(category: VecCategory, max = 1): VecPolicy {
  return { category, max };
}

// A reviewed upper bound, not a target: removing an entry's Vec use is always allowed.
const retainedAllocVec = new Map<string, VecPolicy>([
  ["crates/vize_davinci/src/diagnostic.rs", policy("analysis")],
  ["crates/vize_davinci/src/folio/croquis.rs", policy("contract")],
  ["crates/vize_davinci/src/folio/croquis/parse.rs", policy("contract")],
  ["crates/vize_davinci/src/folio/croquis/parse/entry.rs", policy("contract")],
  ["crates/vize_davinci/src/folio/croquis/print.rs", policy("contract")],
  ["crates/vize_davinci/src/folio/dump.rs", policy("contract")],
  ["crates/vize_davinci/src/folio/feed.rs", policy("contract")],
  ["crates/vize_davinci/src/folio/page.rs", policy("contract")],
  ["crates/vize_davinci/src/pass/pipeline.rs", policy("analysis")],
  ["crates/vize_davinci/src/side_table.rs", policy("analysis", 2)],
  ["crates/vize_disegno/src/expr/filter.rs", policy("analysis", 2)],
  ["crates/vize_disegno/src/folio.rs", policy("contract")],
  ["crates/vize_disegno/src/folio/owned.rs", policy("contract")],
  ["crates/vize_disegno/src/folio/owned/binding.rs", policy("contract")],
  ["crates/vize_disegno/src/folio/parse.rs", policy("contract")],
  ["crates/vize_disegno/src/folio/parse/binding_line.rs", policy("contract")],
  ["crates/vize_disegno/src/folio/parse/line.rs", policy("contract", 12)],
  ["crates/vize_disegno/src/scope.rs", policy("analysis")],
  ["crates/vize_disegno/src/verify.rs", policy("analysis")],
  ["crates/vize_disegno/src/verify/walk.rs", policy("analysis")],
  ["crates/vize_ricalco/src/emit.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/buf.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/component.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/create_slots.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/directive.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/hoist.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/merge.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/model.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/on.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/props.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/props_object.rs", policy("emit")],
  ["crates/vize_ricalco/src/emit/slots.rs", policy("emit")],
  ["crates/vize_ricalco/src/lower.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/binding.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/cx.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/directive.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/element.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/forop.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/structural.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/structural/wrapper.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/sugar.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/text.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/text/condense.rs", policy("lower")],
  ["crates/vize_ricalco/src/lower/vfor.rs", policy("lower", 5)],
  ["crates/vize_ricalco/src/pass/hoist/lattice.rs", policy("pass")],
  ["crates/vize_ricalco/src/pass/legacy/ids.rs", policy("pass")],
  ["crates/vize_ricalco/src/pass/text.rs", policy("pass")],
  ["crates/vize_ricalco/src/pass/vfor.rs", policy("pass")],
  ["crates/vize_ricalco/src/pass/vif.rs", policy("pass")],
  ["crates/vize_ricalco/src/pass/vmodel.rs", policy("pass")],
  ["crates/vize_ricalco/src/pass/vmodel/check.rs", policy("pass")],
  ["crates/vize_ricalco/src/pass/vslot.rs", policy("pass")],
  ["crates/vize_ricalco/src/pass/vslot/consume.rs", policy("pass")],
  ["crates/vize_ricalco/src/pass/vslot/group.rs", policy("pass")],
  ["crates/vize_ricalco/src/pass/vslot/spell.rs", policy("pass")],
]);

function rustFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const absolute = path.join(root, entry.name);
    if (entry.isDirectory()) files.push(...rustFiles(absolute));
    else if (entry.isFile() && entry.name.endsWith(".rs")) files.push(absolute);
  }
  return files;
}

function maskNonCode(source: string): string {
  let output = "";
  let index = 0;
  while (index < source.length) {
    const pair = source.slice(index, index + 2);
    if (pair === "//") {
      const newline = source.indexOf("\n", index + 2);
      if (newline === -1) break;
      output += "\n";
      index = newline + 1;
      continue;
    }
    if (pair === "/*") {
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        const nested = source.slice(index, index + 2);
        if (nested === "/*") {
          depth += 1;
          index += 2;
        } else if (nested === "*/") {
          depth -= 1;
          index += 2;
        } else {
          if (source[index] === "\n") output += "\n";
          index += 1;
        }
      }
      continue;
    }

    const raw = source.slice(index).match(/^(?:br|r)(#*)"/u);
    if (raw) {
      const close = `"${raw[1]}`;
      const contentStart = index + raw[0].length;
      const closeAt = source.indexOf(close, contentStart);
      const end = closeAt === -1 ? source.length : closeAt + close.length;
      output += source.slice(index, end).replace(/[^\n]/gu, " ");
      index = end;
      continue;
    }

    const stringStart = source[index] === '"' ? 1 : pair === 'b"' ? 2 : 0;
    if (stringStart > 0) {
      let escaped = false;
      index += stringStart;
      while (index < source.length) {
        const char = source[index];
        if (char === "\n") output += "\n";
        index += 1;
        if (char === '"' && !escaped) break;
        escaped = char === "\\" && !escaped;
        if (char !== "\\") escaped = false;
      }
      continue;
    }

    output += source[index];
    index += 1;
  }
  return output;
}

function usesDirectStdStorage(source: string): boolean {
  const code = maskNonCode(source);
  return [
    /\bstd\s*::\s*string\s*::\s*String\b/u,
    /\bstd\s*::\s*vec\s*::\s*Vec\b/u,
    /\bstd\s*::\s*(?:prelude\s*::\s*(?:v1|rust_\d+)\s*::\s*)?(?:String|Vec)\b/u,
    /\bstd\s*::\s*collections\s*::\s*(?:HashMap|HashSet|hash_map|hash_set)\b/u,
    /\bstd\s*::\s*collections\s*::\s*\{[^;]*(?:HashMap|HashSet|hash_map|hash_set)\b/u,
    /\bstd\s*::\s*\{[^;]*(?:String|Vec|string\s*::\s*String|vec\s*::\s*Vec|collections\s*::[^;]*(?:HashMap|HashSet|hash_map|hash_set))\b/u,
  ].some((pattern) => pattern.test(code));
}

function isDavinciOptHostEdge(relative: string): boolean {
  return relative.startsWith(davinciOptRoot);
}

const sources = libraryRoots.flatMap((root) => rustFiles(path.join(repoRoot, root)));

test("Davinci stage libraries do not name std storage types", () => {
  const violations = sources
    .map((file) => [path.relative(repoRoot, file), fs.readFileSync(file, "utf8")] as const)
    .filter(([relative, source]) => !isDavinciOptHostEdge(relative) && usesDirectStdStorage(source))
    .map(([relative]) => relative);
  assert.deepEqual(
    violations,
    [],
    `use vize_s0 storage types outside the davinci-opt host edge:\n${violations.join("\n")}`,
  );
});

test("retained alloc Vec sites stay inside their reviewed bounds", () => {
  const violations: string[] = [];
  const usedCategories = new Set<VecCategory>();
  for (const file of sources) {
    const relative = path.relative(repoRoot, file);
    if (isDavinciOptHostEdge(relative)) continue;
    const code = maskNonCode(fs.readFileSync(file, "utf8"));
    const count = code.match(/\balloc\s*::\s*vec\s*::\s*Vec\b/gu)?.length ?? 0;
    if (count === 0) continue;
    const entry = retainedAllocVec.get(relative);
    if (!entry) violations.push(`${relative}: ${count} unclassified use(s)`);
    else if (count > entry.max) {
      violations.push(`${relative}: ${count} use(s), reviewed maximum ${entry.max}`);
    } else {
      usedCategories.add(entry.category);
    }
  }
  for (const category of usedCategories) assert.ok(categoryReasons[category].length > 0);
  assert.deepEqual(
    violations,
    [],
    `review alloc Vec growth or use SmallVec:\n${violations.join("\n")}`,
  );
});

test("the boundary recognizers cover nested imports and the host exception", () => {
  assert.equal(usesDirectStdStorage("use std::{ vec::Vec, collections::{HashMap} };"), true);
  assert.equal(usesDirectStdStorage("use std::prelude::v1::Vec;"), true);
  assert.equal(usesDirectStdStorage("use std::{String, Vec};"), true);
  assert.equal(usesDirectStdStorage("// use std::vec::Vec;\nuse vize_s0::SmallVec;"), false);
  assert.equal(usesDirectStdStorage('let path = "std::vec::Vec";'), false);
  assert.equal(usesDirectStdStorage('let marker = "//"; use std::vec::Vec;'), true);
  assert.equal(isDavinciOptHostEdge("crates/vize_davinci/src/bin/davinci-opt/main.rs"), true);
  assert.equal(isDavinciOptHostEdge("crates/vize_davinci/src/lib.rs"), false);
});
