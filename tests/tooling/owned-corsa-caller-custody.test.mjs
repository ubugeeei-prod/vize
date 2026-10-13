import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  verifyOwnedCorsaCallers,
  ownedCorsaCallerPaths,
} from "../../tools/support/compat/github/owned-corsa-caller-custody.mjs";

const source = (file) => fs.readFileSync(new URL("../../" + file, import.meta.url));
const actual = new Map(
  ownedCorsaCallerPaths.map((file) => {
    const body = source(file);
    return [
      file,
      {
        mode: "100644",
        body,
        blob: createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
      },
    ];
  }),
);
const metadata = JSON.parse(
  fs.readFileSync(
    new URL("../../tools/support/compat/github/owned-corsa-caller-metadata.json", import.meta.url),
  ),
);
const before = new Map(
  metadata.files.map((entry) => [
    entry.file,
    entry.actualBefore
      ? {
          mode: entry.actualBefore.mode,
          blob: entry.actualBefore.blob,
          body: Buffer.from(entry.actualBefore.rawBase64, "base64"),
        }
      : null,
  ]),
);
const reader = (side, file) =>
  side === "after" ? (actual.get(file) ?? null) : (before.get(file) ?? null);
void test("complete physical current two-caller bodies recover whole authenticated historical8304 originals after actual248/cd7 full incoming Reader proof", () => {
  const value = verifyOwnedCorsaCallers(reader);
  assert.equal(value.receipt.completeCallerBodies, 2);
  assert.equal(value.receipt.runtimeCredit, false);
  for (const entry of metadata.files) {
    assert.deepEqual(
      value.reader("after", entry.file).body,
      Buffer.from(entry.historicalAfter.rawBase64, "base64"),
    );
    assert.deepEqual(
      value.reader("before", entry.file)?.body ?? null,
      entry.historicalBefore ? Buffer.from(entry.historicalBefore.rawBase64, "base64") : null,
    );
  }
});
void test("missing, changed, mixed historical and wrong-mode/blob caller bodies refuse before projection", () => {
  const previous = verifyOwnedCorsaCallers(reader);
  for (const file of ownedCorsaCallerPaths) {
    const body = actual.get(file);
    for (const changed of [
      null,
      { ...body, mode: "100755" },
      { ...body, mode: "120000" },
      { ...body, blob: "f".repeat(40) },
      { ...body, body: Buffer.concat([body.body, Buffer.from(" ")]) },
      previous.reader("after", file),
    ])
      assert.throws(() =>
        verifyOwnedCorsaCallers((side, name) =>
          side === "after" && name === file ? changed : reader(side, name),
        ),
      );
  }
});
void test("returned historical metadata/buffers cannot mutate retained caller authority", () => {
  const value = verifyOwnedCorsaCallers(reader);
  const first = value.reader("after", ownedCorsaCallerPaths[0]);
  const body = Buffer.from(first.body);
  first.body.fill(0);
  first.blob = "0".repeat(40);
  first.mode = "100755";
  assert.deepEqual(value.reader("after", ownedCorsaCallerPaths[0]).body, body);
  assert.equal(value.reader("after", ownedCorsaCallerPaths[0]).mode, "100644");
});

void test("actual incoming before-side Reader bodies refuse missing/changed/mode/blob before any historical view", () => {
  for (const file of ownedCorsaCallerPaths) {
    const value = before.get(file);
    const changes = value
      ? [
          null,
          { ...value, mode: "100755" },
          { ...value, blob: "f".repeat(40) },
          { ...value, body: Buffer.concat([value.body, Buffer.from(" ")]) },
        ]
      : [{ mode: "100644", blob: "f".repeat(40), body: Buffer.from("foreign") }];
    for (const changed of changes)
      assert.throws(() =>
        verifyOwnedCorsaCallers((side, name) =>
          side === "before" && name === file ? changed : reader(side, name),
        ),
      );
  }
});
