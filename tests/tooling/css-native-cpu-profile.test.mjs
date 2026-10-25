import assert from "node:assert/strict";
import { test } from "node:test";
import { readFileSync } from "node:fs";
import {
  sampledEventCount,
  readPerfAck,
  samplingPlan,
  validateSamplePacket,
} from "../../tools/benchmarks/scripts/css-native-cpu-profile.mjs";

await test("real perf ACK frames permit partial FIFO reads and reject malformed or truncated frames", () => {
  const consume = (chunks) =>
    readPerfAck((buffer) => {
      const chunk = Buffer.from(chunks.shift() ?? "");
      chunk.copy(buffer);
      return chunk.length;
    });
  for (const chunks of [["ack\n"], ["a", "ck\n"], ["a", "c", "k", "\n"], ["ac", "k\n"]]) {
    const expected = [...chunks].map((value) => Buffer.from(value).toString("hex"));
    assert.deepEqual(consume([...chunks]), { ack: "ack\n", chunks: expected });
  }
  for (const chunks of [[], ["a"], ["a\n"], ["ack!"], ["bad\n"], ["ack\nextra"]])
    assert.throws(() => consume([...chunks]));
});

await test("CPU samples preserve each fixed vector and both operation-specific ABBA role orders", () => {
  const plan = samplingPlan();
  assert.equal(plan.length, 24);
  for (const id of ["two-rules", "400-rules", "400-bindings"]) {
    for (const operation of ["authored", "processed"]) {
      const entries = plan.filter((item) => item.id === id && item.operation === operation);
      assert.deepEqual(
        entries.map((item) => item.side),
        ["base", "head", "head", "base"],
      );
      assert.ok(entries.every((item) => item.calls === (id === "two-rules" ? 500000 : 10000)));
    }
  }
  assert.equal(
    plan.reduce((count, item) => count + item.calls, 0),
    4160000,
  );
});

await test("sampling requires actual event rows and never converts errors or empty output into samples", () => {
  assert.equal(sampledEventCount(" 123.456:\n 125.678:\n"), 2);
  for (const output of ["", "\n", "Operation not permitted", "0 samples", "123.456:\nfailed"])
    assert.throws(() => sampledEventCount(output));
});

await test("sample workload custody compares complete published output and exact actual work", () => {
  const published = JSON.parse(
    readFileSync(
      new URL("../fixtures/css/compile-source-maps/public-0.571.0.json", import.meta.url),
      "utf8",
    ),
  ).captures[0];
  const item = samplingPlan()[0];
  const config = {
    images: { base: { producer: { side: "base", source: "pinned-source" } } },
    rows: [
      {
        id: item.id,
        source: published.source,
        processed: published.source,
        options: published.options,
        expected: { authored: published.output },
      },
    ],
  };
  const packet = {
    status: "complete",
    plan: item,
    producer: config.images.base.producer,
    source: published.source,
    options: published.options,
    before: structuredClone(published.output),
    after: structuredClone(published.output),
    calls: item.calls,
    codeUnits: item.calls * published.output.code.length,
  };
  validateSamplePacket(packet, config, item);
  for (const change of [
    (value) => {
      value.status = "attempt";
    },
    (value) => {
      value.producer.source = "foreign";
    },
    (value) => {
      value.source += " ";
    },
    (value) => {
      value.options.sourceMap = true;
    },
    (value) => {
      value.before.code += " ";
    },
    (value) => {
      value.after.map = "invented";
    },
    (value) => {
      value.calls--;
    },
    (value) => {
      value.codeUnits--;
    },
  ]) {
    const changed = structuredClone(packet);
    change(changed);
    assert.throws(() => validateSamplePacket(changed, config, item));
  }
  const debugConfig = { ...config, kind: "debug-source" };
  assert.throws(() => validateSamplePacket(packet, debugConfig, item));
  const controlled = {
    ...packet,
    samplingControl: [
      { command: "enable", ack: "ack\n" },
      { command: "disable", ack: "ack\n" },
    ],
  };
  validateSamplePacket(controlled, debugConfig, item);
  for (const samplingControl of [
    [],
    controlled.samplingControl.slice(0, 1),
    [{ command: "enable", ack: "invented" }, controlled.samplingControl[1]],
  ])
    assert.throws(() =>
      validateSamplePacket({ ...controlled, samplingControl }, debugConfig, item),
    );
});
