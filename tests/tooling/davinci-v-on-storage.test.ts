import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  V_ON_CORPUS_DIR,
  V_ON_CORPUS_REGEN,
  classify,
  modifiedOnSpellings,
  renderVOnCorpus,
  syntheticBoundary,
  trackedNaturalSources,
} from "../../legacy-tools/davinci/lib/v-on-corpus.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type Buckets = { options: number; event: number; keys: number };
type InventoryFile = { relPath: string; text: string };

function committedInventory(): Map<string, string> {
  const dir = path.join(repoRoot, V_ON_CORPUS_DIR);
  const files = fs.existsSync(dir) ? fs.readdirSync(dir).sort() : [];
  return new Map(
    files.map((name) => [
      `${V_ON_CORPUS_DIR}/${name}`,
      fs.readFileSync(path.join(dir, name), "utf8"),
    ]),
  );
}

test("the natural committed v-on corpus fits the two-entry inline buckets", () => {
  const sources = trackedNaturalSources();
  const spellings = sources.flatMap(({ source }) => modifiedOnSpellings(source));
  const dynamicEntryFixture = sources.find(
    ({ file }) => file === "crates/vize_s1_to_s2/tests/emit_create_slots/dynamic_entries.rs",
  );
  const maxima = spellings.map(classify).reduce<Buckets>(
    (max, buckets) => ({
      options: Math.max(max.options, buckets.options),
      event: Math.max(max.event, buckets.event),
      keys: Math.max(max.keys, buckets.keys),
    }),
    { options: 0, event: 0, keys: 0 },
  );

  assert.ok(dynamicEntryFixture, "the dynamic slot entry parity fixture remains tracked");
  assert.ok(
    modifiedOnSpellings(dynamicEntryFixture.source).includes("@click.prevent"),
    "the Mealie-shaped dynamic slot fixture keeps its natural modified v-on spelling",
  );
  assert.deepEqual(classify("@click.prevent"), { options: 0, event: 1, keys: 0 });
  assert.deepEqual(maxima, { options: 2, event: 2, keys: 2 });

  // The intentional-update tripwire: every natural spelling is recorded, per
  // source file, in the committed per-area inventory, and the live scan must
  // equal it byte for byte (a new, moved, or removed spelling fails until the
  // inventory is regenerated and its diff reviewed). The inventory carries no
  // total, so fixtures added in different areas never conflict.
  const expected: InventoryFile[] = renderVOnCorpus(sources);
  assert.deepEqual(
    [...committedInventory().entries()],
    expected.map((file) => [file.relPath, file.text]),
    `update the measured corpus evidence intentionally: ${V_ON_CORPUS_REGEN}`,
  );
  const inventoried = expected.reduce(
    (count, file) => count + file.text.trimEnd().split("\n").length - 1,
    0,
  );
  assert.equal(inventoried, spellings.length);
});

test("the inventory recognizes both static modified v-on attribute spellings", () => {
  assert.deepEqual(
    modifiedOnSpellings(`
      <button title="1 > 0" @click.stop="go" v-on:keyup.enter.prevent='go'>save</button>
      <Panel\n  @update:modelValue.once="save"\n/>
    `),
    ["@click.stop", "v-on:keyup.enter.prevent", "@update:modelValue.once"],
  );
});

test("the inventory reads escaped inline fixtures carried by source files", () => {
  const embedded = String.raw`const template = "<button @click.stop=\"go\" v-on:keyup.enter=\"go\">";`;
  assert.deepEqual(modifiedOnSpellings(embedded), ["@click.stop", "v-on:keyup.enter"]);
});

test("the inventory rejects non-attribute lookalikes", () => {
  assert.deepEqual(
    modifiedOnSpellings(`
      Contact dev@click.stop or install @scope/pkg.mod.
      macro_rules! route { (@click.stop) => {} }
      const token = "@click.stop";
      <p>text @click.stop</p>
      <div title="please use @click.stop here" data-example="v-on:keyup.enter">x</div>
      <button @[event].stop="dynamic names are outside the static classifier" />
    `),
    [],
  );
});

test("the corpus inventory excludes marked synthetic storage boundaries", () => {
  const source = `<button @click.stop="natural" />
// v-on-storage-synthetic:start
<button @click.stop.prevent.self="synthetic" />
// v-on-storage-synthetic:end`;
  assert.deepEqual(modifiedOnSpellings(source.replace(syntheticBoundary, "")), ["@click.stop"]);
});
