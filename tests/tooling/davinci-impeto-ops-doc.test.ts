import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...segments), "utf8");
}

function rustMnemonics(): string[] {
  const rust = readRepoFile("crates", "vize_impeto", "src", "op", "kind.rs");
  return [
    ...new Set(
      [...rust.matchAll(/Self::[A-Za-z]+ => "(impeto\.[^"]+)"/gu)].map(
        (match) => match[1],
      ),
    ),
  ];
}

function leanParserMnemonics(): string[] {
  const syntax = readRepoFile("formal", "impeto", "Impeto", "Syntax.lean");
  return [...syntax.matchAll(/\| "(impeto\.[^"]+)" => some \.[A-Za-z]+/gu)].map(
    (match) => match[1],
  );
}

function tableRows(): Map<string, { vapor: string; vdom: string }> {
  const doc = readRepoFile("davinci-road", "plan", "impeto-ops.md");
  const rows = new Map<string, { vapor: string; vdom: string }>();

  for (const line of doc.split("\n")) {
    const match = line.match(
      /^\| `(impeto\.[^`]+)` \| .* \| `([^`]+)` \| `([^`]+)` \| .* \|$/u,
    );
    if (match) {
      rows.set(match[1], { vdom: match[2], vapor: match[3] });
    }
  }

  return rows;
}

function leanTraceLabels(functionName: "interpretVDom" | "interpretVapor"): string[] {
  const semantics = readRepoFile("formal", "impeto", "Impeto", "Semantics.lean");
  const start = semantics.indexOf(`def ${functionName}`);
  assert.notEqual(start, -1, `${functionName} is missing`);
  const end = semantics.indexOf("\ndef ", start + 1);
  const block = semantics.slice(start, end === -1 ? undefined : end);
  return [...block.matchAll(/\| \.[A-Za-z]+ => "([^"]+)"/gu)].map((match) => match[1]);
}

test("P3-5 Impeto op reference covers the Rust mnemonic set exactly once", () => {
  const expected = rustMnemonics();
  const rows = tableRows();

  assert.deepEqual([...rows.keys()], expected);
  assert.deepEqual(leanParserMnemonics(), expected);
});

test("P3-5 Impeto op reference records the executable Lean trace labels", () => {
  const rows = [...tableRows().values()];

  assert.deepEqual(
    rows.map((row) => row.vdom),
    leanTraceLabels("interpretVDom"),
  );
  assert.deepEqual(
    rows.map((row) => row.vapor),
    leanTraceLabels("interpretVapor"),
  );
});

test("P3-5 Impeto op reference is cross-linked from Folio docs and rustdoc", () => {
  const folio = readRepoFile("davinci-road", "plan", "folio-format-impeto.md");
  const lib = readRepoFile("crates", "vize_impeto", "src", "lib.rs");
  const phase = readRepoFile("davinci-road", "plan", "phase-3.md");

  assert.match(folio, /\[`impeto-ops\.md`\]\(\.\/impeto-ops\.md\)/u);
  assert.match(lib, /\[`davinci-road\/plan\/impeto-ops\.md`\]/u);
  assert.match(phase, /P3-5 Impeto op reference doc/u);
  assert.match(phase, /\[P3-5 record\]\(\.\/phase-3-records\/p3-5\.md\)/u);
});
