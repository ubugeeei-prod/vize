// Inert protocol laws only; these grant no native or installed execution credit.
import assert from "node:assert/strict";
import { test } from "node:test";
import { validateNativeJournal } from "./support/n8n-installed-cli.ts";

const expected = {
  installRoot: "/inert/public-install",
  nativePath: "/inert/public-install/node_modules/@vizejs/native-darwin-arm64/vize.node",
  nativeSha256: "1".repeat(64),
  node: "/inert/node",
  argv: ["/inert/node", "/inert/public-install/node_modules/vize/bin/vize", "lint"],
  pid: 1234,
  status: 1,
  signal: null,
  corsaPath: "/inert/public-install/node_modules/@typescript/typescript-darwin-arm64/lib/tsc",
};
function journal(): Array<Record<string, unknown>> {
  const common = { schema: "vize-public-native-custody-event-v1", pid: expected.pid };
  return [
    {
      ...common,
      event: "initialized",
      installRoot: expected.installRoot,
      nativePath: expected.nativePath,
      sha256: expected.nativeSha256,
      node: expected.node,
      argv: expected.argv,
    },
    {
      ...common,
      event: "attempt",
      expectedNative: true,
      actualPath: expected.nativePath,
      sha256: expected.nativeSha256,
      corsaPath: expected.corsaPath,
    },
    {
      ...common,
      event: "returned",
      expectedNative: true,
      actualPath: expected.nativePath,
      sha256: expected.nativeSha256,
      corsaPath: expected.corsaPath,
    },
  ];
}
const encode = (events: Array<Record<string, unknown>>) =>
  events.map((event) => JSON.stringify(event)).join("\n") + "\n";
const exit = (code: number) => ({
  schema: "vize-public-native-custody-event-v1",
  pid: expected.pid,
  event: "exit",
  code,
});

test("an observed lint exit one can end after its successful pinned native return", () => {
  const events = journal();
  const before = structuredClone(events);
  const raw = encode(events);
  validateNativeJournal(raw, expected);
  assert.deepEqual(events, before, "no synthetic exit event is added");
  assert.equal(raw, encode(before), "complete original NDJSON stays unchanged");
  assert.deepEqual(
    raw
      .trimEnd()
      .split("\n")
      .map((row) => JSON.parse(row).event),
    ["initialized", "attempt", "returned"],
  );
});

test("ordinary Node exit packets still require a unique final code matching spawn", () => {
  for (const status of [0, 1]) {
    validateNativeJournal(encode([...journal(), exit(status)]), { ...expected, status });
    assert.throws(() =>
      validateNativeJournal(encode([...journal(), exit(1 - status)]), { ...expected, status }),
    );
  }
  assert.throws(() => validateNativeJournal(encode([...journal(), exit(1), exit(1)]), expected));
  assert.throws(() =>
    validateNativeJournal(encode([...journal(), exit(1), journal()[2]]), expected),
  );
});

test("missing hook exit never credits success, an abnormal status, a signal or spawn error", () => {
  const raw = encode(journal());
  for (const result of [
    { status: 0 },
    { status: 2 },
    { status: null },
    { signal: "SIGTERM" },
    { error: { code: "ETIMEDOUT", message: "spawn timeout" } },
  ]) {
    assert.throws(() => validateNativeJournal(raw, { ...expected, ...result } as typeof expected));
  }
  assert.throws(() =>
    validateNativeJournal(raw, { ...expected, signal: undefined } as unknown as typeof expected),
  );
});

test("native nonzero termination keeps exact complete loader and process rejection laws", () => {
  const mutations: Array<(events: Array<Record<string, unknown>>) => void> = [
    (events) => {
      events.shift();
    },
    (events) => {
      events.splice(1, 1);
    },
    (events) => {
      events.pop();
    },
    (events) => {
      events[2].event = "failed";
    },
    (events) => {
      events[2].event = "rejected";
    },
    (events) => {
      events[2].pid = 4321;
    },
    (events) => {
      events[0].argv = ["/inert/node", "workspace-vize", "lint"];
    },
    (events) => {
      events[0].node = "/inert/other-node";
    },
    (events) => {
      events[2].actualPath = "/inert/workspace/other.node";
    },
    (events) => {
      events[2].sha256 = "2".repeat(64);
    },
    (events) => {
      events[2].corsaPath = "/inert/workspace/tsgo";
    },
    (events) => {
      events[2].expectedNative = false;
    },
    (events) => {
      events[2].expectedNative = undefined;
    },
    (events) => {
      events.unshift(events[0]);
    },
    (events) => {
      events.splice(2, 0, events[1]);
    },
    (events) => {
      events.push(events[2]);
    },
    (events) => {
      [events[1], events[2]] = [events[2], events[1]];
    },
    (events) => {
      events.push({ ...events[2], expectedNative: false });
    },
    (events) => {
      events.push({ ...events[2], expectedNative: false, actualPath: "/inert/foreign.node" });
    },
    (events) => {
      events.push({ ...events[1], expectedNative: false, actualPath: "/inert/foreign.node" });
    },
    (events) => {
      events.push(exit(0));
    },
  ];
  for (const mutate of mutations) {
    const events = journal();
    mutate(events);
    assert.throws(() => validateNativeJournal(encode(events), expected));
  }
  assert.throws(() => validateNativeJournal(encode(journal()).trimEnd(), expected));
});
