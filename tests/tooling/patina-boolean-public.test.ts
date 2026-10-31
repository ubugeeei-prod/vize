import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { asArray, asRecord, asString } from "../../tools/support/json-value.ts";
import { assertAncestry } from "../../tools/support/release/patina_boolean_public/authority.ts";
import {
  assertHost,
  assertJournalPid,
  assertReport,
  assertTextImage,
  descendants,
  originalFiles,
  overrideNames,
  readOriginal,
  rejectOverrides,
  requiredMerge,
} from "../../tools/support/release/patina_boolean_public/contract.ts";

const original = readOriginal();
test("11147 whole original authored bytes and complete report remain pinned", () => {
  const files = originalFiles(original);
  assert.equal(files.length, 3);
  assert.equal(
    files[2]?.[1],
    readFileSync(
      new URL(
        "../_fixtures/differential/linter/native-boolean-tsrs-11147/App.vue.txt",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  assert.equal(
    asString(original.originalInstall),
    "pnpm add -D vize@0.608.0 @vizejs/vite-plugin@0.608.0 vite-plus@0.3.0 vue@3.5.43 typescript\n",
  );
  const expected = Buffer.from(JSON.stringify(original.expectedNullableReport));
  assertReport(original, "stable-user-provider", expected);
  assertReport(original, "reported-tsrs", expected);
  assert.throws(
    () => assertReport(original, "original-no-provider", expected),
    /complete ordered original report/u,
  );
  assertReport(
    original,
    "original-no-provider",
    Buffer.from(JSON.stringify(original.expectedNoProviderReport)),
  );
});
test("11147 full report refuses omissions, warning downgrades, changed ranges, extra findings and actual normalization", () => {
  const cases = [
    (rows: unknown[]) => {
      asRecord(rows[0]).file = "App.vue";
    },
    (rows: unknown[]) => {
      asRecord(rows[0]).errorCount = 0;
    },
    (rows: unknown[]) => {
      asRecord(rows[0]).warningCount = 1;
    },
    (rows: unknown[]) => {
      asRecord(asArray(asRecord(rows[0]).messages)[0]).severity = 1;
    },
    (rows: unknown[]) => {
      asRecord(asArray(asRecord(rows[0]).messages)[0]).column = 1;
    },
    (rows: unknown[]) => {
      asRecord(asArray(asRecord(rows[0]).messages)[0]).endColumn = 18;
    },
    (rows: unknown[]) => {
      asRecord(asArray(asRecord(rows[0]).messages)[0]).message = "nullable";
    },
    (rows: unknown[]) => {
      delete asRecord(asArray(asRecord(rows[0]).messages)[0]).ruleDocsPath;
    },
    (rows: unknown[]) => {
      asArray(asRecord(rows[0]).messages).push({ ruleId: "type/corsa-runtime" });
    },
    (rows: unknown[]) => {
      rows.push({ file: "unrelated.vue", messages: [] });
    },
  ];
  for (const mutate of cases) {
    const rows = structuredClone(asArray(original.expectedNullableReport));
    mutate(rows);
    assert.throws(
      () => assertReport(original, "reported-tsrs", Buffer.from(JSON.stringify(rows))),
      /complete ordered original report/u,
    );
  }
});
test("11147 every runtime override and non-original host is refused", () => {
  rejectOverrides({});
  assertHost("darwin", "arm64", "v26.6.0");
  for (const name of overrideNames)
    assert.throws(
      () => rejectOverrides({ [name]: "/foreign/runtime" }),
      /ambient runtime\/source override/u,
    );
  assert.throws(() => assertHost("linux", "arm64", "v26.6.0"), /original macOS/u);
  assert.throws(() => assertHost("darwin", "x64", "v26.6.0"), /original ARM64/u);
  assert.throws(() => assertHost("darwin", "arm64", "v24.14.0"), /Node26/u);
});
test("11147 live checker selection requires its own descendant PID and exact text image", () => {
  assert.deepEqual(
    descendants(
      "1 0 launchd\n20 1 node\n30 20 /app/node\n40 30 /app/node_modules/tsrs\n50 1 /foreign/tsrs\n",
      20,
    ),
    [
      { pid: 30, ppid: 20, command: "/app/node" },
      { pid: 40, ppid: 30, command: "/app/node_modules/tsrs" },
    ],
  );
  assert.throws(() => descendants("20 1 node\n20 9 fake\n", 20), /duplicate ps process identity/u);
  assert.throws(() => descendants("bad record\n", 20), /ps record is malformed/u);
  const image = "/app/node_modules/@tsrs/darwin-arm64/tsrs";
  assertTextImage("p40\nn" + image + "\nn/usr/lib/libSystem.B.dylib\n", 40, image);
  assert.throws(
    () => assertTextImage("p50\nn" + image + "\n", 40, image),
    /observed descendant PID/u,
  );
  assert.throws(
    () => assertTextImage("p40\nn" + image + ".other\n", 40, image),
    /selected loaded text image/u,
  );
  assert.throws(() => assertTextImage("p40\nn" + image + "\np50\nn/other\n", 40, image));
  assertJournalPid(Buffer.from('{"pid":30,"event":"initialized"}\n'), 30);
  assert.throws(
    () => assertJournalPid(Buffer.from('{"pid":40,"event":"initialized"}\n'), 30),
    /independently captured CLI PID/u,
  );
});
test("11147 accepts only a genuine Runtime-containing producer ancestry", () => {
  const valid = {
    base_commit: { sha: requiredMerge },
    merge_base_commit: { sha: requiredMerge },
    behind_by: 0,
    status: "ahead",
  };
  assertAncestry(valid, requiredMerge);
  for (const wrong of [
    { ...valid, base_commit: { sha: "976269c24fd46dd67711b50332df6cd96dcd2c31" } },
    { ...valid, merge_base_commit: { sha: "976269c24fd46dd67711b50332df6cd96dcd2c31" } },
    { ...valid, behind_by: 1 },
    { ...valid, status: "diverged" },
  ])
    assert.throws(() => assertAncestry(wrong, requiredMerge));
});
