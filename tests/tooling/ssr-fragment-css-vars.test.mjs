import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { test } from "node:test";

const capture = process.env.VIZE_FRAGMENT_CSS_CAPTURE;
test("#7892 retains the complete original reported App.vue bytes", () => {
  const original = fs.readFileSync(
    new URL(
      "../_fixtures/differential/compiler/ssr-fragment-css-vars/Reported.vue.txt",
      import.meta.url,
    ),
  );
  assert.equal(original.length, 141);
  assert.equal(
    createHash("sha256").update(original).digest("hex"),
    "07d4314a95dedd77e9f972831fbc6969f2ab43cc403fc44da15f64877a7407af",
  );
});
test(
  "#7892 actual original whole SSR/client/fallback modules and production hydration",
  {
    skip: !capture && "source-bound runtime runs in the scoped SSR composite action",
  },
  async () => {
    assert.ok(capture);
    const destination = process.env.VIZE_FRAGMENT_CSS_RUNTIME_CAPTURE;
    assert.ok(destination, "retain complete runtime observations");
    const { observeFragmentCss } = await import("./support/ssr-fragment-css-vars.mjs");
    const result = await observeFragmentCss(JSON.parse(fs.readFileSync(capture, "utf8")));
    assert.equal(result.cases.length, 10);
    fs.writeFileSync(destination, JSON.stringify(result));
  },
);
