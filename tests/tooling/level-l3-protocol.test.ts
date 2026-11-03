import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
// Existing tags are immutable. The explicit child-value operation appends tag
// 16 rather than reusing display-text semantics or renumbering an older op.
const opcodeContract = [
  ["SetProp", 0, "l3.set-prop"],
  ["SetDynamicProps", 1, "l3.set-dynamic-props"],
  ["SetText", 2, "l3.set-text"],
  ["SetEvent", 3, "l3.set-event"],
  ["SetHtml", 4, "l3.set-html"],
  ["SetTemplateRef", 5, "l3.set-template-ref"],
  ["InsertNode", 6, "l3.insert-node"],
  ["PrependNode", 7, "l3.prepend-node"],
  ["Directive", 8, "l3.directive"],
  ["If", 9, "l3.if"],
  ["For", 10, "l3.for"],
  ["CreateComponent", 11, "l3.create-component"],
  ["SlotOutlet", 12, "l3.slot-outlet"],
  ["GetTextChild", 13, "l3.get-text-child"],
  ["ChildRef", 14, "l3.child-ref"],
  ["NextRef", 15, "l3.next-ref"],
  ["SetChildValue", 16, "l3.set-child-value"],
] as const;
const mnemonics = opcodeContract.map(([, , mnemonic]) => mnemonic);

test("the active Rust and Lean graph readers bind the new explicit protocol", () => {
  const rust = fs.readFileSync(path.join(repoRoot, "davinci/vize_l3/src/dump.rs"), "utf8");
  const lean = fs.readFileSync(
    path.join(repoRoot, "tests/formal/impeto/Impeto/Folio.lean"),
    "utf8",
  );
  assert.match(rust, /#\[dump\(name = "l3-dump-v2"\)\]/u);
  for (const section of ["", ".regions", ".ops", ".edges", ".effects"]) {
    assert.ok(lean.includes(`"[l3-dump-v2${section}]"`));
  }
  assert.ok(!rust.includes("s3-folio"));
  assert.ok(!lean.includes("s3-folio"));
});

test("the canonical printer and parser preserve all sixteen old tags and append only child-value tag sixteen", () => {
  const rust = fs.readFileSync(path.join(repoRoot, "davinci/vize_l3/src/op/kind.rs"), "utf8");
  const printed = [...rust.matchAll(/Self::([A-Za-z]+) => "(l3\.[^"]+)"/gu)].map((match) => [
    match[1],
    match[2],
  ]);
  const parsed = [...rust.matchAll(/b"(l3\.[^"]+)" => Some\(Self::([A-Za-z]+)\)/gu)].map(
    (match) => [match[2], match[1]],
  );
  const tags = [...rust.matchAll(/^    ([A-Za-z]+) = (\d+),$/gmu)].map((match) => [
    match[1],
    Number(match[2]),
  ]);
  assert.deepEqual(
    tags,
    opcodeContract.map(([variant, tag]) => [variant, tag]),
  );
  assert.deepEqual(
    printed,
    opcodeContract.map(([variant, , mnemonic]) => [variant, mnemonic]),
  );
  assert.deepEqual(parsed, printed);
});

test("all active generated formal graph fixtures use the current protocol and known opcodes", () => {
  const fixtureRoot = path.join(repoRoot, "tests/formal/impeto/fixtures");
  const rust = fs.readFileSync(path.join(repoRoot, "davinci/vize_l3/src/op/kind.rs"), "utf8");
  const names = new Set(
    [...rust.matchAll(/Self::[A-Za-z]+ => "(l3\.[^"]+)"/gu)].map((match) => match[1]),
  );
  assert.deepEqual([...names], mnemonics);
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
