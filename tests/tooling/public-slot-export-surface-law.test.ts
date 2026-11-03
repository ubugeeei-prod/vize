import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertPublicSlotExportSurface,
  type ProvenDefaultChange,
} from "./support/public-slot-export-surface.ts";

const surface = (defaultText: string | null) => ({
  module: "vize/virtual-ts",
  exports: [
    {
      name: "$Example",
      type: true,
      value: false,
      namespace: false,
      declarations: [
        {
          kind: "TypeAliasDeclaration",
          parameters: [
            { name: "T", constraint: null, default: null },
            { name: "__K", constraint: null, default: defaultText },
          ],
          signatures: [],
        },
      ],
    },
  ],
});

const change: ProvenDefaultChange = {
  exportName: "$Example",
  parameter: "__K",
  before: "$Old<T>",
  after: "$New<T>",
  oracle: "tests/tooling/public-slot-export-surface-law.test.ts",
};

void test("an unchanged surface passes and an unlisted default change fails", () => {
  assertPublicSlotExportSurface(surface("$Old<T>"), surface("$Old<T>"), []);
  assert.throws(
    () => assertPublicSlotExportSurface(surface("$New<T>"), surface("$Old<T>"), []),
    /retain their kinds and parameters/u,
  );
});

void test("a listed default passes only from its exact old text to its exact new text", () => {
  assertPublicSlotExportSurface(surface("$New<T>"), surface("$Old<T>"), [change]);
  // A third text, a stale entry, a different old text and a missing oracle all fail.
  for (const [actual, expected, listed] of [
    [surface("$Other<T>"), surface("$Old<T>"), change],
    [surface("$Old<T>"), surface("$Old<T>"), change],
    [surface("$New<T>"), surface("$Older<T>"), change],
    [
      surface("$New<T>"),
      surface("$Old<T>"),
      { ...change, oracle: "tests/tooling/missing.test.ts" },
    ],
    [surface("$New<T>"), surface("$Old<T>"), { ...change, parameter: "T" }],
  ] as const) {
    assert.throws(() => assertPublicSlotExportSurface(actual, expected, [listed]));
  }
});
