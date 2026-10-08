import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { browserOriginCurrentReference } from "../differential/browser-origin-current-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = path.join(root, "crates/vize/tests/fixtures/issue-7907");
const source = JSON.parse(fs.readFileSync(path.join(directory, "source.json"), "utf8"));
const bytes = fs.readFileSync(path.join(directory, "current-reference-7908.json"));

await test("one exact current watch reference preserves historic coordinates and producer laws", () => {
  const { row } = browserOriginCurrentReference(root, source);
  assert.deepEqual(row.historicalDoctorLocations, [{ start: 141, end: 141, line: 6, column: 35 }]);
});

await test("forged current output and authority expansion fail closed", () => {
  for (const mutate of [
    (data) => data.cases.push(data.cases[0]),
    (data) => {
      data.cases[0].currentExpected = data.cases[0].historicalExpected;
    },
    (data) => {
      data.cases[0].historicalDoctorLocations = [];
    },
  ]) {
    const data = JSON.parse(bytes);
    mutate(data);
    assert.throws(() =>
      browserOriginCurrentReference(root, source, Buffer.from(JSON.stringify(data))),
    );
  }
});

await test("same-name forged historical expectation cannot acquire current-reference authority", () => {
  const forged = structuredClone(source);
  forged.cases.find((entry) => entry.path === "PageTitle.vue").expected[0].messages = [];
  assert.throws(() => browserOriginCurrentReference(root, forged));
});
