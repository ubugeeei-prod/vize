import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { frozenVaporSsrExposePair } from "../_helpers/vapor-ssr-expose-successor.ts";
import { assertComposedVaporSfcStylePackets } from "./support/vapor-style-successor-composition.ts";

type Rows = Parameters<typeof assertComposedVaporSfcStylePackets>[0];
const read = (side: "base" | "head"): Rows =>
  JSON.parse(
    fs.readFileSync(
      new URL(
        "../_fixtures/differential/compiler/vapor-literal-object-styles-10749/authored-module-successor-10701/" +
          side +
          "-sfc.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as Rows;

test("both admissions accept the same complete four original and successor packets", () => {
  assertComposedVaporSfcStylePackets(read("base"), read("head"));
  for (const index of [1, 3]) {
    const pair = frozenVaporSsrExposePair(index);
    assert.deepEqual(read("base")[index], pair.before);
    assert.deepEqual(read("head")[index], pair.after);
  }
});

test("omitting any complete packet on either side is rejected", () => {
  for (const side of ["base", "head"] as const) {
    for (let index = 0; index < 4; index++) {
      const before = read("base"),
        after = read("head");
      (side === "base" ? before : after).splice(index, 1);
      assert.throws(() => assertComposedVaporSfcStylePackets(before, after));
    }
  }
});

test("duplicating a complete packet on either side is rejected", () => {
  for (const side of ["base", "head"] as const) {
    const before = read("base"),
      after = read("head");
    const rows = side === "base" ? before : after;
    const first = rows[0];
    assert.ok(first);
    rows.push(structuredClone(first));
    assert.throws(() => assertComposedVaporSfcStylePackets(before, after));
  }
});

test("counterfeit code, output fields and source fields are rejected on both sides", () => {
  for (const side of ["base", "head"] as const) {
    for (let index = 0; index < 4; index++) {
      for (const facet of ["code", "extra", "source"] as const) {
        const before = read("base"),
          after = read("head");
        const row = (side === "base" ? before : after)[index];
        assert.ok(row);
        if (facet === "code") row.output.code += "\n/* unregistered */";
        else if (facet === "extra") row.output.unregistered = true;
        else row.source += "\n<!-- unregistered -->";
        assert.throws(() => assertComposedVaporSfcStylePackets(before, after));
      }
    }
  }
});
