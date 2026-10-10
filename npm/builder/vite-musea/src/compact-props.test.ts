import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";
import { analyzeSfcFallback } from "./native-loader.ts";
import { compactPropsFixture } from "./compact-props-fixtures.ts";

const fixture = await compactPropsFixture();
for (const vector of fixture.cases) {
  void test(`direct type-literal props: ${vector.name}`, () => {
    assert.deepEqual(analyzeSfcFallback(vector.source), vector.expected);
  });
}

void test("original compact and multiline bytes retain equivalent complete analysis", () => {
  const [compact, multiline] = fixture.cases;
  assert.equal(
    createHash("sha256").update(compact.source).digest("hex"),
    fixture.originalCompactSourceSha256,
  );
  assert.equal(
    createHash("sha256").update(multiline.source).digest("hex"),
    fixture.originalMultilineSourceSha256,
  );
  assert.deepEqual(analyzeSfcFallback(compact.source), analyzeSfcFallback(multiline.source));
});
