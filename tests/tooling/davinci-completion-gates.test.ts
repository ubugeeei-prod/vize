import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  codenames,
  evaluate,
  gates,
  type Repo,
  scanCodenames,
  workspaceMembers,
} from "../../tools/commands/davinci/completion-gates.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const doc = fs.readFileSync(path.join(root, "docs/davinci/completion-gates.md"), "utf8");

const memory = (files: Record<string, string>): Repo => ({
  paths: Object.keys(files),
  read: (file) => files[file] ?? null,
});

void test("the document and the tool define the same gates in the same order", () => {
  const rows = [...doc.matchAll(/^\| `([a-z]+\.[a-z0-9-]+)` +\|.*\| (measured|issue) +\|/gm)];
  assert.deepEqual(
    rows.map((row) => row[1]),
    gates.map((gate) => gate.id),
  );
  for (const [, id, kind] of rows) {
    const gate = gates.find((entry) => entry.id === id);
    assert.equal(kind === "measured", gate?.measure !== undefined, id);
  }
});

void test("every gate names an issue and the tier order never goes back", () => {
  let tier = 0;
  for (const gate of gates) {
    assert.ok(gate.issues.length > 0, gate.id);
    assert.ok(gate.tier >= tier, gate.id);
    tier = gate.tier;
  }
});

void test("codename scan counts paths and lines but not aliases, history or the docs that may keep them", () => {
  // Spelled through the gate's own list so this file holds no banned literal.
  const pick = (prefix: string) => {
    const found = codenames.find((entry) => entry.startsWith(prefix));
    if (!found) throw new Error(prefix);
    return found;
  };
  const [l3, dump, stages, program] = [pick("imp"), pick("fol"), pick("spo"), pick("dav")];
  const title = (word: string) => word[0].toUpperCase() + word.slice(1);
  const report = scanCodenames(
    memory({
      "levels/vize_l3/src/lib.rs": `//! L3 - reactivity IR (codename: ${title(l3)})\npub struct ${title(dump)};\n`,
      [`crates/vize_curator/tests/${stages}_payload.rs`]: "fn main() {}\n",
      "levels/vize_l2/src/lib.rs": `//! L2 - the ${title(program)} IR (codename: ${title(l3)})\n`,
      "crates/vize/src/profile.rs": `record("${program}.expr.parses");\n`,
      [`docs/${program}/plan/${dump}-format.md`]: `${title(dump)} and ${title(program)}\n`,
      "docs/content/guide.md": `${title(l3)} and ${title(program)}\n`,
      "CHANGELOG.md": l3 + "\n",
      [`tools/commands/${program}/completion-gates.ts`]: JSON.stringify(l3) + ",\n",
      "tests/_fixtures/run/cargo.jsonl": `{"${l3}": 1}\n`,
      "tests/_fixtures/differential/compiler/case/observed-base.json": `"${program}.x.y"\n`,
    }),
  );
  assert.equal(report.paths, 1);
  assert.equal(report.lines, 4);
  assert.deepEqual(report.byName, { [stages]: 1, [dump]: 1, [program]: 3 });
});

void test("measured gates fail on a legacy edge and pass once it is gone", () => {
  const cargo = (members: string[]) =>
    `[workspace]\nmembers = [\n${members.map((m) => `  "${m}",`).join("\n")}\n]\n`;
  const levels = (members: string[]) =>
    gates
      .find((gate) => gate.id === "levels.crates")
      ?.measure?.(memory({ "Cargo.toml": cargo(members) }));
  const home = (dir: string) =>
    [
      "vize_l0",
      "vize_l0_derive",
      "vize_l1",
      "vize_l1_to_l2",
      "vize_l2",
      "vize_l2_to_l3",
      "vize_l3",
      "vize_l4",
    ].map((name) => `${dir}/${name}`);
  assert.equal(levels(home("levels"))?.pass, true);
  const extra = levels([...home("levels"), "levels/vize_substrate", "crates/vize_croquis"]);
  assert.equal(extra?.pass, false);
  assert.match(extra?.detail ?? "", /extra \[vize_substrate\]/);

  const products = (entries: unknown[]) =>
    gates
      .find((gate) => gate.id === "products.dependencies")
      ?.measure?.(
        memory({
          "docs/davinci/plan/product-dependency-allowlists/vize_glyph.json": JSON.stringify({
            entries,
          }),
        }),
      );
  assert.equal(products([{ from: "vize_glyph", to: "vize_atelier_sfc" }])?.pass, false);
  assert.equal(products([])?.pass, true);
});

void test("issue gates are unknown offline and fail while an issue is open", () => {
  const states = new Map<number, "OPEN" | "CLOSED">(
    gates.flatMap((gate) => gate.issues).map((n) => [n, "CLOSED"]),
  );
  states.set(6858, "OPEN");
  const issueGates = gates.filter((gate) => gate.measure === undefined);
  const tsrx = (results: ReturnType<typeof evaluate>) =>
    results.find((result) => result.id === "inputs.tsrx");
  assert.equal(tsrx(evaluate(memory({}), null, issueGates))?.status, "unknown");
  const online = evaluate(memory({}), states, issueGates);
  assert.equal(online.filter((result) => result.status === "pass").length, issueGates.length - 1);
  assert.equal(tsrx(online)?.status, "fail");
  assert.deepEqual(tsrx(online)?.openIssues, [6858]);
});

void test("workspace members are read from the real Cargo.toml", () => {
  const members = workspaceMembers(fs.readFileSync(path.join(root, "Cargo.toml"), "utf8"));
  assert.ok(members.includes("davinci/vize_l0"));
});
