import assert from "node:assert/strict";
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
