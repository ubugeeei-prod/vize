import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { listenerWrapperReference } from "./listener-wrapper-reference.js";
import { publicInstanceReference } from "./public-instance-reference.js";

const files = ["basic", "updates-and-language", "art", "feature-boundaries"];
const archived = () =>
  publicInstanceReference(
    files.flatMap((file) =>
      JSON.parse(
        fs.readFileSync(new URL(`./expected-packets/${file}.json`, import.meta.url), "utf8"),
      ),
    ),
  );
const source = fs.readFileSync(
  new URL("./reference-fixtures/listener-wrapper.vue.txt", import.meta.url),
  "utf8",
);
const listener = (packets) =>
  packets.find((row) => row.name === "feature-boundary-diagnostics").result[0];

void test("listener reference advances exactly four expected fields and preserves rename", () => {
  const before = archived();
  const untouched = structuredClone(before);
  const out = listenerWrapperReference(before);
  const expected = structuredClone(before);
  const value = "(value: number) => undefined";
  for (const diagnostic of [listener(expected), listener(expected).relatedInformation[0]]) {
    diagnostic.start = source.indexOf(value);
    diagnostic.length = value.length;
  }
  assert.deepEqual(out, expected);
  assert.deepEqual(before, untouched);
  assert.deepEqual(out.at(-1), before.at(-1), "unexecuted rename remains the old expectation");
  assert.equal(out.length, 38);
});

void test("bad main and related spans fail their semantic bounds before the image hash", () => {
  const bad = archived();
  listener(bad).start++;
  assert.throws(() => listenerWrapperReference(bad), /closed historical listener start/);
  const related = archived();
  listener(related).relatedInformation[0].length--;
  assert.throws(() => listenerWrapperReference(related), /closed historical related length/);
});

void test("unknown fields fail the closed diagnostic schema", () => {
  const bad = archived();
  listener(bad).future = true;
  assert.throws(() => listenerWrapperReference(bad), /closed listener diagnostic fields/);
  const related = archived();
  listener(related).relatedInformation[0].future = true;
  assert.throws(() => listenerWrapperReference(related), /closed related diagnostic fields/);
});

void test("modified source and unrelated packet changes cannot reuse this transition", () => {
  assert.throws(
    () => listenerWrapperReference(archived(), source + "\n"),
    /closed authored listener fixture/,
  );
  const bad = archived();
  bad[0].result[0].messageText += " future";
  assert.throws(() => listenerWrapperReference(bad), /closed historical packet image/);
  assert.throws(
    () => listenerWrapperReference(listenerWrapperReference(archived())),
    /closed historical listener start/,
  );
});
