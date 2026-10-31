import assert from "node:assert/strict";
import { test } from "node:test";

import {
  collectVizeDiagnostics,
  collectVueTscDiagnostics,
} from "../../tools/support/compat/fixtures/typecheck-divergence-input.ts";

const diagnostic = "error:3:7 [TS2322] Type  string is not assignable (source: value)";
const expected = {
  diagnostics: [
    {
      file: "src/App.vue",
      severity: "error",
      line: 3,
      column: 7,
      code: 2322,
      message: "Type string is not assignable",
    },
  ],
  excludedNonVueCount: 0,
};

test("typed collectors preserve inherited and array-shaped diagnostic envelopes", () => {
  const file = { file: "src/App.vue", diagnostics: [diagnostic] };
  const inherited: unknown = Object.create({ files: [file] });
  const arrayEnvelope = Object.assign([], { files: [file] });
  const arrayFile = Object.assign([], file);
  assert.deepEqual(collectVizeDiagnostics(inherited, "/fixture"), expected);
  assert.deepEqual(collectVizeDiagnostics(arrayEnvelope, "/fixture"), expected);
  assert.deepEqual(collectVizeDiagnostics({ files: [arrayFile] }, "/fixture"), expected);
});

test("typed collectors retain envelope getter order and complete range refusals", () => {
  const accesses: string[] = [];
  const file = {
    get file() {
      accesses.push("file");
      return "src/App.vue";
    },
    get diagnostics() {
      accesses.push("diagnostics");
      return [diagnostic];
    },
  };
  const report = {
    get files() {
      accesses.push("files");
      return [file];
    },
  };
  assert.deepEqual(collectVizeDiagnostics(report, "/fixture"), expected);
  assert.deepEqual(accesses, ["files", "files", "diagnostics", "file", "diagnostics"]);
  for (const value of [
    "0:1 [TS2322]",
    "1:0 [TS2322]",
    "1:1 [TS0]",
    "9007199254740992:1 [TS2322]",
  ]) {
    assert.throws(
      () =>
        collectVizeDiagnostics(
          { files: [{ file: "src/App.vue", diagnostics: [`error:${value} invalid`] }] },
          "/fixture",
        ),
      {
        name: "Error",
        message:
          "Invalid typecheck divergence input: diagnostic range and code must be positive safe integers: src/App.vue",
      },
    );
  }
});

test("baseline support filtering retains the complete separate exclusion counts", () => {
  assert.deepEqual(
    collectVueTscDiagnostics(
      [
        "src/App.vue(1,1): error TS2322: authored",
        "docs/.vitepress/Support.vue(1,1): error TS2322: support",
        ".Local.vue(1,1): error TS2322: authored dot filename",
        "src/types.ts(1,1): error TS2322: non-Vue",
        "../external.vue(1,1): error TS2322: external",
        "error TS6059: project configuration",
      ].join("\n"),
      "/fixture",
      { comparableVueFiles: new Set(["src/App.vue"]) },
    ),
    {
      diagnostics: [
        {
          file: "src/App.vue",
          severity: "error",
          line: 1,
          column: 1,
          code: 2322,
          message: "authored",
        },
        {
          file: ".Local.vue",
          severity: "error",
          line: 1,
          column: 1,
          code: 2322,
          message: "authored dot filename",
        },
      ],
      excludedNonVueCount: 1,
      excludedProjectCount: 1,
      excludedExternalCount: 1,
      excludedSupportVueCount: 1,
    },
  );
});
