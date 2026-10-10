import assert from "node:assert/strict";
import test from "node:test";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { variantComponentNames } from "./variant-bindings.ts";

const plan = (names: string[]) => variantComponentNames(names.map((name) => ({ name })));

void test("ordinary variant bindings retain their exact legacy names", () => {
  assert.deepEqual(
    [...plan(["Default", "State enabled", "日本語", "42", "!"])],
    [
      ["Default", "Default"],
      ["State enabled", "StateEnabled"],
      ["日本語", "日本語"],
      ["42", "Variant42"],
      ["!", "Variant"],
    ],
  );
});

for (const names of [
  ["State enabled", "State-enabled", "State: enabled"],
  ["Café", "Cafe", "Cafe\u0301"],
  ["!", "?", "Variant"],
  ["42", "Variant42"],
  ["日本 語", "日本-語"],
  ["\uD800", "\uD801", "\uFFFD"],
]) {
  void test(`normalized collisions stay distinct: ${JSON.stringify(names)}`, () => {
    const bindings = plan(names);
    assert.equal(new Set(bindings.values()).size, names.length);
    for (const name of names) assert.match(bindings.get(name)!, /^__MuseaVariant_[a-f0-9]{64}$/);
    assert.deepEqual(
      [...plan([...names].reverse())].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)),
      [...bindings].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)),
    );
    assert.equal(plan([...names, "Unrelated"]).get("Unrelated"), "Unrelated");
    for (const name of names)
      assert.equal(plan([...names, "Unrelated"]).get(name), bindings.get(name));
  });
}

void test("plans reflect membership changes and reject duplicate authored names", () => {
  const variants = [{ name: "State enabled" }];
  assert.equal(variantComponentNames(variants).get(variants[0].name), "StateEnabled");
  variants.push({ name: "State-enabled" });
  assert.match(variantComponentNames(variants).get(variants[0].name)!, /^__MuseaVariant_/);
  variants.pop();
  assert.equal(variantComponentNames(variants).get(variants[0].name), "StateEnabled");
  assert.throws(() => plan(["Default", "Default"]), /Duplicate Musea variant name: "Default"/);
  assert.deepEqual([...plan([])], []);
});

void test("the genuine public red and ordinary control retain their authenticated authored bytes", async () => {
  const hashes = {
    "Host.vue": "3087f1c9c435eccf08fedd48050fd124ab5574ac5c85f508b6b561ea5a0e599c",
    "Host.art.vue": "903b7d320984de800c38d489dd08bbb8e97fbb06a32bd0f85f324529016a14a3",
    "ordinary/Host.vue": "3087f1c9c435eccf08fedd48050fd124ab5574ac5c85f508b6b561ea5a0e599c",
    "ordinary/Host.art.vue": "be8a636e93e8635a0edb608c4be0e4b0c148ad6bd6849820822e7c47b855a228",
  };
  for (const [file, expected] of Object.entries(hashes)) {
    const bytes = await readFile(
      new URL(
        `../../../../tests/_fixtures/differential/musea/variant-binding-collision/${file}`,
        import.meta.url,
      ),
    );
    assert.equal(createHash("sha256").update(bytes).digest("hex"), expected);
  }
});
