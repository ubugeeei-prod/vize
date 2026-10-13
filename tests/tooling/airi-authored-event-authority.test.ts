import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { appendAiriAuthoredEventSuccessor } from "./support/airi-authored-event-authority.ts";
import type { FixtureProject } from "./support/real-project-lsp-report.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registry = JSON.parse(
  fs.readFileSync(path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
) as { projects: Array<FixtureProject & { repository: string }> };
const airi = registry.projects.find((project) => project.id === "airi")!;
const items = airi.lspAuthoredOracle!.componentBoundary.completionItems;

test("Airi's seventh pinned event owner preserves every original ranked completion", () => {
  assert.deepEqual(appendAiriAuthoredEventSuccessor(airi, items.slice(0, 40)), items);
  assert.deepEqual(items.slice(40), [
    { label: "@activate", rank: 40 },
    { label: "@delete", rank: 41 },
    { label: "@select", rank: 42 },
  ]);
});

test("Airi authority refuses a changed predecessor bank or source owner", () => {
  const original = items.slice(0, 40);
  for (const changed of [
    original.slice(0, 39),
    [...original, { label: "@phantom", rank: 40 }],
    original.map((item, index) => (index === 39 ? { ...item, rank: 38 } : item)),
    original.map((item, index) => (index === 2 ? { ...item, label: "v-phantom" } : item)),
  ]) {
    assert.throws(() => appendAiriAuthoredEventSuccessor(airi, changed));
  }
  assert.throws(() =>
    appendAiriAuthoredEventSuccessor({ ...airi, revision: "0".repeat(40) }, original),
  );
  const changedOwner = structuredClone(airi);
  changedOwner.lspAuthoredOracle!.componentBoundary.componentFile = "Other.vue";
  assert.throws(() => appendAiriAuthoredEventSuccessor(changedOwner, original));
});

test("finite Airi authority never changes another project's ranked bank", () => {
  const other = registry.projects.find((project) => project.id === "pinia")!;
  const original = other.lspAuthoredOracle!.componentBoundary.completionItems;
  assert.equal(appendAiriAuthoredEventSuccessor(other, original), original);
});
