import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import {
  assertNonRuleWitness,
  witnessBlocks,
} from "../../docs/scripts/ja-nonrule-render-assertions.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { currentEditorialWitness } from "../../docs/scripts/config-editorial-succession.ts";

const root = resolve(import.meta.dirname, "../..");
const read = (path: string) => readFileSync(resolve(root, path), "utf8");
const fixture = JSON.parse(read("tests/_fixtures/docs-ja-nonrule-editorial.json"));

test("all whole native witnesses reject original mistranslations and preserve native bookmarks", () => {
  assert.equal(fixture.cases.length, 31);
  for (const original of fixture.cases) {
    const entry = currentEditorialWitness(original);
    const path = `docs/content${entry.route}.md`;
    const source = read(path);
    assert(source.includes(entry.after), `${entry.case}: entire authored successor`);
    assert(!source.includes(entry.before), `${entry.case}: entire erroneous predecessor removed`);
    assertNonRuleWitness(witnessBlocks(entry, "after"), entry);
    assert.throws(() => assertNonRuleWitness(witnessBlocks(entry, "before"), entry));
    if (entry.originalAlias)
      assert.equal(
        source.split(`<span id="${entry.originalAlias}"></span>`).length,
        2,
        `${entry.case}: one original bookmark`,
      );
    assert(
      defaultNavigationRenderRoutes.includes(entry.route),
      `${entry.route}: original full native route campaign`,
    );
  }
});

test("canonical editorial successors preserve both complete frozen generations and current Corsa limits", () => {
  const current = JSON.parse(read("tests/_fixtures/docs-vue-config-editorial-succession.json"));
  const digest = (path: string) => createHash("sha256").update(read(path)).digest("hex");
  assert.equal(current.originalSha256, digest("tests/_fixtures/docs-ja-nonrule-editorial.json"));
  assert.equal(
    current.previousSuccessionSha256,
    digest("tests/_fixtures/docs-config-editorial-succession.json"),
  );
  assert.deepEqual(
    current.witnesses.map((entry: { case: string }) => entry.case),
    ["N01", "N02", "N03", "N04", "N05", "N06", "N07", "N11"],
  );
  const native = read("crates/vize/src/commands/check/runner/resolve.rs");
  const maximum = native.match(/const MAX_CORSA_SERVERS: usize = (\d+);/)?.[1];
  assert.equal(maximum, "32", "the new worker-limit description follows the real native contract");
  assert.match(native, /if servers == 0/);
  assert.match(native, /if servers > MAX_CORSA_SERVERS/);
  const workers = current.witnesses.find((entry: { case: string }) => entry.case === "N07");
  assert.match(workers.previous, /`1` のみ/);
  assert.match(workers.after, new RegExp("`1`〜`" + maximum + "`"));
  const model = read("davinci/vize_l0/src/config/model.rs");
  assert.match(model, /raw_type_checker\.options_api\.unwrap_or\(true\)/);
});
