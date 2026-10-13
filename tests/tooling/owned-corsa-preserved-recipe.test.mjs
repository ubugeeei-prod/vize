// Locked-source qualification controls only; never native or performance acceptance.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { ensureHostedLiteralSource } from "../performance/support/warm-type-backed-git-reader-literal-source.ts";
import {
  qualifyOwnedCorsaPreservedRecipe,
  preservedRecipeDigest,
} from "../performance/support/warm-type-backed-owned-corsa-recipe.ts";
const file = "pnpm-lock.yaml";
const root =
  process.env.OWNED_RECIPE_SOURCE_ROOT ?? fs.realpathSync(new URL("../../", import.meta.url));
assert(root, "an actual owned raw source root is required for this source law");
const sha = (body) => createHash("sha256").update(body).digest("hex");
const cell = (revision) => {
  const entry = execFileSync(
    "git",
    ["--no-replace-objects", "ls-tree", "-z", revision, "--", file],
    { cwd: root },
  ).toString();
  const match = /^100644 blob ([a-f0-9]{40})\tpnpm-lock.yaml\0$/u.exec(entry);
  assert(match);
  const body = execFileSync("git", ["--no-replace-objects", "cat-file", "blob", match[1]], {
    cwd: root,
    maxBuffer: 4 * 1024 * 1024,
  });
  return { mode: "100644", blob: match[1], body };
};
for (const revision of [
  "a93e78147b0990c813e965a7868b32dda341eb77",
  "437c7f8cba66dc9517b8c01659063143185747ea",
  "b2caebeab6fb5bc817b4e31a65d5d560449b691b",
])
  ensureHostedLiteralSource(root, revision);
const before = cell("a93e78147b0990c813e965a7868b32dda341eb77");
const after = cell("437c7f8cba66dc9517b8c01659063143185747ea");
const historical = cell("b2caebeab6fb5bc817b4e31a65d5d560449b691b");
const read = (b, a) => (side, requested) => {
  assert.equal(requested, file);
  return side === "before" ? b : a;
};
void test("both complete authentic incoming raw locks retain the literal old source cell", () => {
  const recipe = qualifyOwnedCorsaPreservedRecipe(read(before, after));
  assert.equal(recipe.actualBeforeSHA256, sha(before.body));
  assert.equal(recipe.actualAfterSHA256, sha(after.body));
  const calls = [];
  const digest = preservedRecipeDigest(recipe, (side, path) => {
    calls.push([side, path]);
    return "unchanged";
  });
  assert.equal(digest("before", file), sha(historical.body));
  assert.equal(digest("after", file), sha(historical.body));
  assert.equal(digest("after", "Cargo.toml"), "unchanged");
  assert.deepEqual(calls, [["after", "Cargo.toml"]]);
  assert.equal(Object.isFrozen(recipe), true);
  assert.equal(Reflect.set(recipe, "retainedHistoricalSHA256", "forged"), false);
  assert.equal(digest("after", file), sha(historical.body));
  assert.equal(recipe.runtimeCredit, false);
  assert.equal(recipe.performanceCredit, false);
});
void test("the complete unchanged historical class follows the original qualification", () => {
  assert.equal(qualifyOwnedCorsaPreservedRecipe(read(historical, historical)), null);
  assert.equal(preservedRecipeDigest(null, () => "original")("before", file), "original");
});
void test("mixed, missing, partial, mutated, wrong-blob and executable lock cells refuse", () => {
  for (const [b, a] of [
    [before, historical],
    [historical, after],
    [null, after],
    [before, null],
    [{ ...before, body: before.body.subarray(1) }, after],
    [before, { ...after, body: Buffer.concat([after.body, Buffer.from(" ")]) }],
    [{ ...before, blob: "0".repeat(40) }, after],
    [before, { ...after, mode: "100755" }],
  ])
    assert.throws(() => qualifyOwnedCorsaPreservedRecipe(read(b, a)));
});
