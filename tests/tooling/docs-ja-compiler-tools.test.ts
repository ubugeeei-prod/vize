import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { missingInboundAnchors } from "./support/docs-inbound-anchors.ts";

const root = resolve(import.meta.dirname, "../..");
const read = (path: string) => readFileSync(resolve(root, path), "utf8");
const verdictNames: Record<string, string> = {
  Same: "equivalent",
  Equivalent: "equivalent",
  Todo: "deferred",
  Deferred: "deferred",
  Divergent: "divergent",
};

/** Corpus row → documented verdict word, read from the Rust oracle table. */
function rustVerdicts(): Map<string, string> {
  const source = read("crates/vize_atelier_jsx/tests/babel_compat/verdicts.rs");
  const verdicts = new Map<string, string>();
  for (const [, row, variant] of source.matchAll(
    /\(\s*"([a-z_]+\/[a-z0-9_]+)",\s*(?:Verdict::)?([A-Z][A-Za-z]*)/g,
  )) {
    assert.ok(variant in verdictNames, `${row}: unknown verdict ${variant}`);
    verdicts.set(row, verdictNames[variant]);
  }
  assert.ok(verdicts.size > 50, "the verdict table was parsed");
  return verdicts;
}

void test("Babel compatibility pages agree with the Rust verdict table", () => {
  const verdicts = rustVerdicts();
  const categories = new Set([...verdicts.keys()].map((row) => row.split("/")[0]));
  const deferred = [...verdicts]
    .filter(([, verdict]) => verdict === "deferred")
    .map(([row]) => row);
  for (const locale of ["", "ja/"]) {
    const page = `docs/content/${locale}guide/jsx-babel-compat.md`;
    const source = read(page);
    const tableRows = Array.from(
      source.matchAll(/^\| `([a-z_]+\/[a-z0-9_]+)` \|/gm),
      (match) => match[1],
    );
    for (const row of deferred) assert.ok(tableRows.includes(row), `${page}: ${row} is tabulated`);
    for (const row of tableRows)
      assert.notEqual(
        verdicts.get(row) ?? "equivalent",
        "equivalent",
        `${page}: ${row} still differs`,
      );
    for (const line of source.split("\n")) {
      const rows = Array.from(line.matchAll(/`([a-z_]+)\/([a-z0-9_]+)`/g))
        .filter(([, category]) => categories.has(category))
        .map(([, category, name]) => `${category}/${name}`);
      for (const row of rows) assert.ok(verdicts.has(row), `${page}: ${row} is a corpus row`);
      if (rows.length !== 1) continue;
      for (const word of ["equivalent", "deferred", "divergent"])
        if (line.includes(`\`${word}\``))
          assert.equal(verdicts.get(rows[0]), word, `${page}: ${rows[0]} is ${word}`);
    }
    for (const [, path] of source.matchAll(
      /\]\(https:\/\/github\.com\/ubugeeei-prod\/vize\/blob\/main\/([^)#\s]+)/g,
    ))
      assert.ok(existsSync(resolve(root, decodeURIComponent(path))), `${page}: ${path} exists`);
  }
});

void test("compiler guide anchors that other pages link to exist", () => {
  for (const name of ["jsx", "jsx-babel-compat", "compiler-inspector"]) {
    for (const locale of ["", "ja/"]) {
      const page = `${locale}guide/${name}.md`;
      assert.deepEqual(missingInboundAnchors(page), [], page);
    }
  }
});
