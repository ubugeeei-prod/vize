import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { formatSfc } from "../../npm/native/index.js";

const fixtures = JSON.parse(
  fs.readFileSync(
    new URL(
      "../_fixtures/differential/formatter/sfc-object-wrap-13666/cases.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  cases: { id: string; printWidth: number; source: string; preserve: string; collapse: string }[];
};

test("native objectWrap preserves or collapses objects with the pinned layout rules", () => {
  for (const fixture of fixtures.cases) {
    const source = `<script setup lang="ts">\n${fixture.source}</script>\n`;
    for (const objectWrap of ["preserve", "collapse"] as const) {
      const options = { objectWrap, printWidth: fixture.printWidth };
      const expected = `<script setup lang="ts">\n${fixture[objectWrap]}</script>\n`;
      const first = formatSfc(source, options);
      assert.equal(first.code, expected, `${fixture.id} ${objectWrap}`);
      assert.deepEqual(formatSfc(first.code, options), { code: expected, changed: false });
    }
    assert.equal(
      formatSfc(source, { printWidth: fixture.printWidth }).code,
      `<script setup lang="ts">\n${fixture.preserve}</script>\n`,
    );
  }
});

test("native objectWrap rejects values outside the public option union", () => {
  assert.throws(
    () => formatSfc("<script>const a={};</script>", { objectWrap: "auto" as "preserve" }),
    { code: "InvalidArg", message: "Unknown objectWrap; expected 'preserve' or 'collapse'" },
  );
});
