import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("the active Rust and Lean graph readers bind the new explicit protocol", () => {
  const rust = fs.readFileSync(path.join(repoRoot, "crates/vize_l3/src/dump.rs"), "utf8");
  const lean = fs.readFileSync(path.join(repoRoot, "tests/formal/l3/L3/Folio.lean"), "utf8");
  assert.match(rust, /#\[dump\(name = "l3-dump-v2"\)\]/u);
  for (const section of ["", ".regions", ".ops", ".edges", ".effects"]) {
    assert.ok(lean.includes(`"[l3-dump-v2${section}]"`));
  }
  assert.ok(!rust.includes("s3-folio"));
  assert.ok(!lean.includes("s3-folio"));
});

test("the canonical printer and parser each cover exactly sixteen opcode names", () => {
  const rust = fs.readFileSync(path.join(repoRoot, "crates/vize_l3/src/op/kind.rs"), "utf8");
  const printed = [...rust.matchAll(/Self::[A-Za-z]+ => "(l3\.[^"]+)"/gu)].map((match) => match[1]);
  const parsed = [...rust.matchAll(/b"(l3\.[^"]+)" => Some\(Self::[A-Za-z]+\)/gu)].map(
    (match) => match[1],
  );
  assert.equal(printed.length, 16);
  assert.equal(new Set(printed).size, 16);
  assert.deepEqual(parsed, printed);
});

test("all active generated formal graph fixtures use the current protocol and known opcodes", () => {
  const fixtureRoot = path.join(repoRoot, "tests/formal/l3/fixtures");
  const rust = fs.readFileSync(path.join(repoRoot, "crates/vize_l3/src/op/kind.rs"), "utf8");
  const names = new Set(
    [...rust.matchAll(/Self::[A-Za-z]+ => "(l3\.[^"]+)"/gu)].map((match) => match[1]),
  );
  assert.equal(names.size, 16);
  let cases = 0;
  for (const filename of fs
    .readdirSync(fixtureRoot)
    .filter((name) => name.endsWith(".lowered.jsonl"))) {
    const content = fs.readFileSync(path.join(fixtureRoot, filename), "utf8");
    for (const line of content.split("\n").filter(Boolean)) {
      const entry = JSON.parse(line);
      assert.match(entry.graph, /^\[l3-dump-v2\]\nphase=built\n/u);
      const opcodes = [...entry.graph.matchAll(/kind=(l3\.[a-z-]+)/gu)].map((match) => match[1]);
      assert.ok(opcodes.length > 0);
      for (const opcode of opcodes) assert.ok(names.has(opcode), opcode);
      assert.ok(!entry.graph.includes("impeto."));
      cases += 1;
    }
  }
  assert.ok(cases > 0);
});
