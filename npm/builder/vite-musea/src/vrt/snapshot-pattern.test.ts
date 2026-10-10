import assert from "node:assert/strict";
import test from "node:test";
import { matchesSnapshotPattern } from "./snapshot-pattern.ts";

void test("approval patterns escape real Art/variant names instead of interpreting them as regex", () => {
  for (const name of [
    "[slug]",
    "(Button)",
    "Button+",
    "Button?",
    "Button$",
    "Button^",
    "Button{2}",
    "Button|Badge",
    "Button.name",
    "名前",
  ]) {
    assert.equal(matchesSnapshotPattern(`routes/${name}/Default`, `routes/${name}/*`), true, name);
    assert.equal(matchesSnapshotPattern(`other/${name}/Default`, `routes/${name}/*`), false, name);
  }
  assert.equal(matchesSnapshotPattern("routes/s/Button/Default", "routes/[slug]/Button/*"), false);
});

void test("single stars stay within a path component while double stars cross directories", () => {
  assert.equal(matchesSnapshotPattern("left/Button/Default", "left/Button/*"), true);
  assert.equal(matchesSnapshotPattern("left/nested/Button/Default", "left/*/Default"), false);
  assert.equal(matchesSnapshotPattern("left/nested/Button/Default", "left/**/Default"), true);
  assert.equal(matchesSnapshotPattern("left/Button/Default", "**"), true);
  assert.equal(matchesSnapshotPattern("left/Button/Default", "Button"), true);
  assert.equal(matchesSnapshotPattern("left/Button/Default", "Badge"), false);
});
