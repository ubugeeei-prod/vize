import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

void test("Nuxt module entry avoids import.meta for Nuxt 2 loaders", () => {
  const source = fs.readFileSync(new URL("./index.ts", import.meta.url), "utf8");

  assert.equal(source.includes("import.meta"), false);
});
