import assert from "node:assert/strict";
import fs from "node:fs";
import { assertVaporSfcExposeTransition } from "../../_helpers/vapor-ssr-expose-successor.ts";
import { assertAuthoredStylePackets } from "./vapor-style-authored-transition.ts";

type Rows = Parameters<typeof assertVaporSfcExposeTransition>[0];
const original = new URL(
  "../../_fixtures/differential/compiler/vapor-literal-object-styles-10749/authored-module-successor-10701/base-sfc.json",
  import.meta.url,
);

/** Authenticate the whole original before applying both unchanged admissions. */
export function assertComposedVaporSfcStylePackets(before: Rows, after: Rows): void {
  assert.deepEqual(
    before,
    JSON.parse(fs.readFileSync(original, "utf8")),
    "all four complete original SFC producer packets remain immutable",
  );
  assertAuthoredStylePackets("sfc", after);
  assertVaporSfcExposeTransition(before, after);
}
