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
import {
  currentEditorialWitness,
  originalEditorialSource,
} from "../../docs/scripts/config-editorial-succession.ts";

const root = resolve(import.meta.dirname, "../..");
const read = (path: string) => readFileSync(resolve(root, path), "utf8");
const sha = (value: string) => createHash("sha256").update(value).digest("hex");
const fixture = JSON.parse(read("tests/_fixtures/docs-ja-nonrule-editorial.json"));

test("Japanese configuration and Musea retain every original complete code block and link", () => {
  assert.equal(fixture.custody.length, 2);
  let count = 0;
  for (const entry of fixture.custody) {
    const source = originalEditorialSource(entry.path, read(entry.path));
    const fences = [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm)].map(
      (match) => match[0],
    );
    count += fences.length;
    assert.equal(fences.length, entry.originalCodeBlocks, entry.path);
    assert.equal(sha(fences.join("\0")), entry.originalWholeCodeSha256, entry.path);
    for (const fence of fences)
      assert(
        read(entry.path).includes(fence),
        `${entry.path}: original complete example remains published`,
      );
    assert.deepEqual(
      [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]),
      entry.originalLinks,
      entry.path,
    );
  }
  assert.equal(count, 28);
});

test("the bounded Japanese changes preserve readable pages and English and measured authority", () => {
  assert.equal(fixture.unchangedAuthority.length, 17);
  for (const entry of fixture.unchangedAuthority)
    assert.equal(
      sha(originalEditorialSource(entry.path, read(entry.path))),
      entry.originalSha256,
      entry.path,
    );
  const backend = fixture.cases.find((entry: { case: string }) => entry.case === "N31");
  assert.equal(fixture.measurementCustody.length, 2);
  for (const entry of fixture.measurementCustody) {
    const source = read(entry.path);
    assert.equal(source.split(backend.after).length, 2, `${entry.path}: one owning output line`);
    assert.equal(
      sha(source.replace(backend.after, backend.before)),
      entry.originalSha256,
      `${entry.path}: all original measurement bytes`,
    );
  }
});

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

test("configuration succession refuses missing, changed, and added editorial bytes", () => {
  const path = "docs/content/guide/configuration-reference.md";
  const source = read(path);
  const original = originalEditorialSource(path, source);
  assert.equal(
    sha(original),
    fixture.unchangedAuthority.find((entry: { path: string }) => entry.path === path)
      .originalSha256,
  );
  for (const changed of [
    source.slice(1),
    source + "\nUnreviewed configuration.\n",
    source.replace("musea", "other"),
  ])
    assert.throws(() => originalEditorialSource(path, changed), /complete configuration successor/);
});
