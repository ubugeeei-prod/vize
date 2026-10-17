import assert from "node:assert/strict";
import fs from "node:fs";
import { sha256 } from "../typed_defaults/custody.mjs";

export const fixtureUrl = new URL(
  "../../../../tests/_fixtures/public/vue-script-suppressions-9291.json",
  import.meta.url,
);
export const fixtureSha256 = "56e707cee8d2ced783a1ff7c6d87e79e12cd5f8d754a1813728b3f9d3b492be5";

export function loadFixtures() {
  const bytes = fs.readFileSync(fixtureUrl);
  assert.equal(
    sha256(bytes),
    fixtureSha256,
    "the public version must not change the frozen packet",
  );
  const fixture = JSON.parse(bytes);
  assert.equal(fixture.schema, "vize.public-vue-script-suppressions-9291");
  assert.equal(fixture.cases.length, 72);
  assert.equal(new Set(fixture.cases.map(({ id }) => id)).size, 72);
  for (const sourceCase of fixture.cases) {
    assert.equal(sha256(sourceCase.source), sourceCase.sourceSha256, sourceCase.id);
    assert.equal(sha256(sourceCase.config), sourceCase.configSha256, sourceCase.id);
    assert.ok(fixture.authority.tests.includes(sourceCase.sourceTest));
  }
  return { fixture, bytes };
}
