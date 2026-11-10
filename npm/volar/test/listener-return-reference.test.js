import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { listenerWrapperReference } from "./listener-wrapper-reference.js";
import {
  beforeListenerReturnReference,
  listenerReturnReference,
} from "./listener-return-reference.js";
import { publicInstanceReference } from "./public-instance-reference.js";

const files = ["basic", "updates-and-language", "art", "feature-boundaries"];
const current = () =>
  files.flatMap((file) =>
    JSON.parse(
      fs.readFileSync(new URL(`./expected-packets/${file}.json`, import.meta.url), "utf8"),
    ),
  );
const listenerMessage = (packets) =>
  packets.find((row) => row.name === "feature-boundary-diagnostics").result[0].messageText
    .messageText;

void test("listener return parity advances one exact diagnostic after historical transitions", () => {
  const packets = current();
  const untouched = structuredClone(packets);
  const historical = beforeListenerReturnReference(packets);
  assert.match(listenerMessage(historical), /=> any'\.$/);
  const successor = listenerReturnReference(
    listenerWrapperReference(publicInstanceReference(packets)),
  );
  assert.match(listenerMessage(successor), /=> void'\.$/);
  assert.deepEqual(packets, untouched);
});

void test("listener return parity rejects unrelated current and successor packet changes", () => {
  const currentMutation = current();
  currentMutation[0].result[0].messageText += " changed";
  assert.throws(() => beforeListenerReturnReference(currentMutation), /closed listener return/);

  const wrappedMutation = listenerWrapperReference(publicInstanceReference(current()));
  wrappedMutation.at(-1).name += "-changed";
  assert.throws(() => listenerReturnReference(wrappedMutation), /closed listener return/);
});
