import assert from "node:assert/strict";
import type { WasmCase } from "./wasm-corpus.ts";

function record(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
function number(value: unknown): number {
  assert.equal(typeof value, "number");
  return value as number;
}

function checkGraph(test: WasmCase, packet: unknown): void {
  const report = record(record(packet).complexityReport);
  const input = record(report.input);
  assert.deepEqual(
    [
      input.reactiveNodeCount,
      input.reactiveEdgeCount,
      input.reactiveCycleCount,
      record(report.dimensions).reactiveGraph,
    ],
    [test.before.nodes, test.before.edges, test.before.cycles, test.before.score],
    `${test.id}: reviewed public observation`,
  );
}

export function expectedWasmPacket(test: WasmCase, before: unknown): unknown {
  checkGraph(test, before);
  const expected: unknown = structuredClone(before);
  const delta = test.after.score - test.before.score;
  assert.equal(
    test.before.nodes,
    test.after.nodes,
    "existing Croquis node facts are outside this effects-only repair",
  );
  if (!delta && test.before.edges === test.after.edges && test.before.cycles === test.after.cycles)
    return expected;
  const packet = record(expected);
  const report = record(packet.complexityReport);
  const input = record(report.input);
  input.reactiveEdgeCount = test.after.edges;
  input.reactiveCycleCount = test.after.cycles;
  record(report.dimensions).reactiveGraph = test.after.score;
  report.totalScore = number(report.totalScore) + delta;
  report.band = test.afterBand;
  assert.ok(Array.isArray(packet.complexityHotspots));
  const hotspots = packet.complexityHotspots
    .map(record)
    .filter((hotspot) => hotspot.fileName === test.child);
  assert.equal(hotspots.length, 1);
  const hotspot = hotspots[0];
  const localInput = record(hotspot.input);
  assert.equal(localInput.reactiveEdgeCount, test.before.edges);
  assert.equal(localInput.reactiveCycleCount, test.before.cycles);
  localInput.reactiveEdgeCount = test.after.edges;
  localInput.reactiveCycleCount = test.after.cycles;
  record(hotspot.dimensions).reactiveGraph = test.after.score;
  hotspot.totalScore = number(hotspot.totalScore) + delta;
  const dominant = record(hotspot.dominantDimension);
  assert.equal(dominant.dimension, "reactive-graph");
  dominant.score = test.after.score;
  return expected;
}
