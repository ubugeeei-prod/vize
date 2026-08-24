import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  categoryReasons,
  expectedInventorySummary,
  retainedAllocVec,
  summarizeInventory,
  type VecMeasurement,
} from "./davinci-storage-inventory.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const libraryRoots = [
  "crates/vize_davinci/src",
  "crates/vize_sinopia/src",
  "crates/vize_disegno/src",
  "crates/vize_ricalco/src",
];
const davinciOptRoot = "crates/vize_davinci/src/bin/davinci-opt/";

function rustFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const absolute = path.join(root, entry.name);
    if (entry.isDirectory()) files.push(...rustFiles(absolute));
    else if (entry.isFile() && entry.name.endsWith(".rs")) files.push(absolute);
  }
  return files;
}

function maskRange(output: string[], start: number, end: number): void {
  for (let index = start; index < end; index += 1) {
    if (output[index] !== "\n" && output[index] !== "\r") output[index] = " ";
  }
}

function charLiteralEnd(source: string, quote: number): number | undefined {
  let cursor = quote + 1;
  if (source[cursor] === "\\") {
    cursor += 1;
    if (source[cursor] === "u" && source[cursor + 1] === "{") {
      const brace = source.indexOf("}", cursor + 2);
      if (brace === -1) return undefined;
      cursor = brace + 1;
    } else if (source[cursor] === "x") {
      cursor += 3;
    } else {
      cursor += 1;
    }
  } else {
    const point = source.codePointAt(cursor);
    if (point === undefined || point === 0x0a || point === 0x0d || source[cursor] === "'") {
      return undefined;
    }
    cursor += point > 0xffff ? 2 : 1;
  }
  return source[cursor] === "'" ? cursor + 1 : undefined;
}

function quotedLiteralEnd(source: string, quote: number): number {
  let escaped = false;
  for (let cursor = quote + 1; cursor < source.length; cursor += 1) {
    const char = source[cursor];
    if (char === '"' && !escaped) return cursor + 1;
    escaped = char === "\\" && !escaped;
    if (char !== "\\") escaped = false;
  }
  return source.length;
}

function maskNonCode(source: string): string {
  const output = source.split("");
  let index = 0;
  while (index < source.length) {
    const pair = source.slice(index, index + 2);
    if (pair === "//") {
      const newline = source.indexOf("\n", index + 2);
      const end = newline === -1 ? source.length : newline;
      maskRange(output, index, end);
      index = end;
      continue;
    }
    if (pair === "/*") {
      let depth = 1;
      let cursor = index + 2;
      while (cursor < source.length && depth > 0) {
        const nested = source.slice(cursor, cursor + 2);
        if (nested === "/*") {
          depth += 1;
          cursor += 2;
        } else if (nested === "*/") {
          depth -= 1;
          cursor += 2;
        } else {
          cursor += 1;
        }
      }
      maskRange(output, index, cursor);
      index = cursor;
      continue;
    }

    const raw = source.slice(index).match(/^(?:br|r)(#*)"/u);
    if (raw) {
      const close = `"${raw[1]}`;
      const contentStart = index + raw[0].length;
      const closeAt = source.indexOf(close, contentStart);
      const end = closeAt === -1 ? source.length : closeAt + close.length;
      maskRange(output, index, end);
      index = end;
      continue;
    }

    const byteCharEnd = pair === "b'" ? charLiteralEnd(source, index + 1) : undefined;
    const charEnd = source[index] === "'" ? charLiteralEnd(source, index) : undefined;
    const literalEnd = byteCharEnd ?? charEnd;
    if (literalEnd !== undefined) {
      maskRange(output, index, literalEnd);
      index = literalEnd;
      continue;
    }

    const stringStart = source[index] === '"' ? 1 : pair === 'b"' ? 2 : 0;
    if (stringStart > 0) {
      const end = quotedLiteralEnd(source, index + stringStart - 1);
      maskRange(output, index, end);
      index = end;
      continue;
    }

    index += 1;
  }
  return output.join("");
}

function usesDirectStdStorage(source: string): boolean {
  const code = maskNonCode(source);
  return [
    /\bextern\s+crate\s+std\b/u,
    /\buse\s+(?:::)?\s*std\s+as\s+[A-Za-z_]\w*\s*;/u,
    /\buse\s+(?:::)?\s*std\s*::\s*(?:vec|collections)\b/u,
    /\buse\s+(?:::)?\s*std\s*::\s*\{[^;]*\b(?:vec|collections)\b/u,
    /\bstd\s*::\s*string\s*::\s*String\b/u,
    /\bstd\s*::\s*vec\s*::\s*Vec\b/u,
    /\bstd\s*::\s*(?:prelude\s*::\s*(?:v1|rust_\d+)\s*::\s*)?(?:String|Vec)\b/u,
    /\bstd\s*::\s*collections\s*::\s*(?:HashMap|HashSet|hash_map|hash_set)\b/u,
    /\bstd\s*::\s*collections\s*::\s*\{[^;]*(?:HashMap|HashSet|hash_map|hash_set)\b/u,
    /\bstd\s*::\s*\{[^;]*(?:String|Vec|string\s*::\s*String|vec\s*::\s*Vec|collections\s*::[^;]*(?:HashMap|HashSet|hash_map|hash_set))\b/u,
  ].some((pattern) => pattern.test(code));
}

function usesOpaqueAllocVecBinding(source: string): boolean {
  const code = maskNonCode(source);
  return [
    /\bextern\s+crate\s+alloc\s+as\b/u,
    /\buse\s+(?:::)?\s*alloc\s+as\b/u,
    /\buse\s+(?:::)?\s*alloc\s*::\s*vec\s*::\s*\{/u,
    /\buse\s+(?:::)?\s*alloc\s*::\s*\{[^;]*\bvec\b/u,
  ].some((pattern) => pattern.test(code));
}

function measureAllocVec(source: string): VecMeasurement {
  const code = maskNonCode(source);
  const bindings = new Set<string>();
  const moduleBindings = new Set<string>();
  const importPattern =
    /\buse\s+(?:::)?\s*alloc\s*::\s*vec\s*::\s*Vec(?:\s+as\s+([A-Za-z_]\w*))?\s*;/gu;
  let withoutImports = code.replace(importPattern, (statement, alias: string | undefined) => {
    bindings.add(alias ?? "Vec");
    return statement.replace(/[^\n\r]/gu, " ");
  });
  const moduleImportPattern = /\buse\s+(?:::)?\s*alloc\s*::\s*vec(?:\s+as\s+([A-Za-z_]\w*))?\s*;/gu;
  withoutImports = withoutImports.replace(
    moduleImportPattern,
    (statement, alias: string | undefined) => {
      moduleBindings.add(alias ?? "vec");
      return statement.replace(/[^\n\r]/gu, " ");
    },
  );
  let boundUses = 0;
  for (const binding of bindings) {
    const usePattern = new RegExp(`(?<![:A-Za-z0-9_])${binding}\\b`, "gu");
    boundUses += withoutImports.match(usePattern)?.length ?? 0;
  }
  for (const binding of moduleBindings) {
    const usePattern = new RegExp(`(?<![:A-Za-z0-9_])${binding}\\s*::\\s*Vec\\b`, "gu");
    boundUses += withoutImports.match(usePattern)?.length ?? 0;
  }
  return {
    directPaths: code.match(/\balloc\s*::\s*vec\s*::\s*Vec\b/gu)?.length ?? 0,
    boundUses,
  };
}

function isDavinciOptHostEdge(relative: string): boolean {
  return relative.startsWith(davinciOptRoot);
}

const sources = libraryRoots.flatMap((root) => rustFiles(path.join(repoRoot, root)));

function measureInventory(): Map<string, VecMeasurement> {
  const measured = new Map<string, VecMeasurement>();
  for (const file of sources) {
    const relative = path.relative(repoRoot, file);
    if (isDavinciOptHostEdge(relative)) continue;
    const actual = measureAllocVec(fs.readFileSync(file, "utf8"));
    if (actual.directPaths > 0 || actual.boundUses > 0) measured.set(relative, actual);
  }
  return measured;
}

function inventoryViolations(measured: ReadonlyMap<string, VecMeasurement>): string[] {
  const violations: string[] = [];
  for (const [relative, actual] of measured) {
    const entry = retainedAllocVec.get(relative);
    if (!entry) {
      violations.push(
        `${relative}: unclassified direct=${actual.directPaths}, bound=${actual.boundUses}`,
      );
    } else if (actual.directPaths !== entry.directPaths || actual.boundUses !== entry.boundUses) {
      violations.push(
        `${relative}: actual direct=${actual.directPaths}, bound=${actual.boundUses}; ` +
          `ledger direct=${entry.directPaths}, bound=${entry.boundUses}`,
      );
    }
  }
  for (const [relative, entry] of retainedAllocVec) {
    if (!measured.has(relative))
      violations.push(`${relative}: stale ${entry.category} ledger entry`);
    assert.ok(categoryReasons[entry.category].length > 0);
  }
  return violations;
}

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

test("alloc Vec bindings stay canonical and auditable", () => {
  const violations = sources
    .map((file) => [path.relative(repoRoot, file), fs.readFileSync(file, "utf8")] as const)
    .filter(
      ([relative, source]) => !isDavinciOptHostEdge(relative) && usesOpaqueAllocVecBinding(source),
    )
    .map(([relative]) => relative);
  assert.deepEqual(
    violations,
    [],
    `bind alloc::vec::Vec directly so its uses can be counted:\n${violations.join("\n")}`,
  );
});

test("retained alloc Vec paths and bound uses equal the reviewed ledger", () => {
  const violations = inventoryViolations(measureInventory());
  assert.deepEqual(
    violations,
    [],
    `update the exact alloc Vec ledger after review:\n${violations.join("\n")}`,
  );
});

test("the executable inventory totals equal the plan record", () => {
  const summary = summarizeInventory(measureInventory());
  assert.deepEqual(summary, expectedInventorySummary);

  const plan = fs.readFileSync(
    path.join(repoRoot, "davinci-road/plan/storage-boundary.md"),
    "utf8",
  );
  const headline = plan.match(
    /contain\s+(\d+) reviewed files,\s+(\d+) direct[^,]+,\s+and\s+(\d+) bound/iu,
  );
  assert.ok(headline, "storage plan must state the executable inventory totals");
  assert.deepEqual(headline.slice(1).map(Number), [
    summary.files,
    summary.directPaths,
    summary.boundUses,
  ]);

  const rows = new Map(
    [
      ...plan.matchAll(
        /^\|\s*(contract|analysis|lower|pass|emit)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|/gmu,
      ),
    ].map(([, category, files, directPaths, boundUses]) => [
      category,
      { files: Number(files), directPaths: Number(directPaths), boundUses: Number(boundUses) },
    ]),
  );
  assert.deepEqual(Object.fromEntries(rows), summary.categories);
});

test("the boundary recognizers cover nested imports and the host exception", () => {
  assert.equal(usesDirectStdStorage("use std::{ vec::Vec, collections::{HashMap} };"), true);
  assert.equal(usesDirectStdStorage("use std::prelude::v1::Vec;"), true);
  assert.equal(usesDirectStdStorage("use std::{String, Vec};"), true);
  assert.equal(usesDirectStdStorage("use std::vec as heap; type Items = heap::Vec<u8>;"), true);
  assert.equal(
    usesDirectStdStorage("use std::collections as maps; type Index = maps::HashMap<u8, u8>;"),
    true,
  );
  assert.equal(
    usesDirectStdStorage("extern crate std as host; type Items = host::vec::Vec<u8>;"),
    true,
  );
  assert.equal(usesDirectStdStorage("use ::std::vec as heap; type Items = heap::Vec<u8>;"), true);
  assert.equal(usesDirectStdStorage("// use std::vec::Vec;\nuse vize_s0::SmallVec;"), false);
  assert.equal(usesDirectStdStorage('let path = "std::vec::Vec";'), false);
  assert.equal(usesDirectStdStorage('let marker = "//"; use std::vec::Vec;'), true);
  assert.equal(usesDirectStdStorage("let quote = '\"'; use std::vec::Vec;"), true);
  assert.equal(usesDirectStdStorage("let quote = b'\"'; use std::collections as maps;"), true);
  assert.equal(usesDirectStdStorage("fn borrow<'a>(value: &'a str) {} use std::vec::Vec;"), true);
  assert.equal(usesOpaqueAllocVecBinding("use alloc::vec as heap;"), false);
  assert.equal(usesOpaqueAllocVecBinding("use alloc::{vec as heap};"), true);
  assert.deepEqual(measureAllocVec("use alloc::vec::Vec as Heap; type Items = Heap<u8>;"), {
    directPaths: 1,
    boundUses: 1,
  });
  assert.deepEqual(measureAllocVec("use alloc::vec as heap; type Items = heap::Vec<u8>;"), {
    directPaths: 0,
    boundUses: 1,
  });
  const reduced = measureInventory();
  const [relative, measurement] = reduced.entries().next().value!;
  reduced.set(relative, { ...measurement, directPaths: measurement.directPaths - 1 });
  assert.match(inventoryViolations(reduced).join("\n"), /actual direct=.*ledger direct=/u);
  reduced.delete(relative);
  assert.match(inventoryViolations(reduced).join("\n"), /stale .* ledger entry/u);
  assert.equal(isDavinciOptHostEdge("crates/vize_davinci/src/bin/davinci-opt/main.rs"), true);
  assert.equal(isDavinciOptHostEdge("crates/vize_davinci/src/lib.rs"), false);
});
