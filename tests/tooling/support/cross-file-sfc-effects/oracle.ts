import assert from "node:assert/strict";
import type { EffectCase } from "./corpus.ts";

function record(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}

export function checkWasm(test: EffectCase, packet: unknown): void {
  const report = record(record(packet).complexityReport);
  const input = record(report.input);
  if (!test.wasmComparable) return; // The retained raw packet documents that separate host limitation.
  assert.deepEqual(
    [
      input.reactiveNodeCount,
      input.reactiveEdgeCount,
      input.reactiveCycleCount,
      record(report.dimensions).reactiveGraph,
      report.totalScore,
      report.band,
    ],
    [
      test.graph.nodes,
      test.graph.edges,
      test.graph.cycles,
      test.graph.score,
      test.total,
      test.band,
    ],
    test.id,
  );
}

function metric(text: string, label: string): number {
  const rows = text.split("\n").filter((line) => line.startsWith(`| ${label} | `));
  assert.equal(rows.length, 1, `one exact ${label} row`);
  const match = rows[0].match(/^\| [^|]+ \| (\d+) \|$/u);
  assert.ok(match, rows[0]);
  return Number(match[1]);
}

function replaceOnce(text: string, before: string, after: string): string {
  assert.equal(text.split(before).length, 2, `one exact reviewed replacement: ${before}`);
  return text.replace(before, after);
}

export function expectedReport(test: EffectCase, before: string): string {
  // This constructs the complete expected successor. Actual source output is
  // never filtered or normalized: all bytes outside these reviewed score and
  // driver fields must stay identical to the frozen public CLI observation.
  assert.equal(metric(before, "reactive-graph"), test.graph.nodes);
  if (!test.graph.edges && !test.graph.cycles) {
    assert.equal(metric(before, "Total score"), test.total);
    return before;
  }
  assert.ok(test.child);
  let expected = replaceOnce(
    before,
    `| reactive-graph | ${test.graph.nodes} |`,
    `| reactive-graph | ${test.graph.score} |`,
  );
  expected = replaceOnce(
    expected,
    `| Total score | ${metric(before, "Total score")} |`,
    `| Total score | ${test.total} |`,
  );
  const component = test.child.replace(/\.vue$/u, "");
  if (test.band !== "low") {
    expected = replaceOnce(expected, "| Band | low |", `| Band | ${test.band} |`);
  }
  const beforeScore = test.graph.nodes + 1;
  const afterScore = test.graph.score + 1;
  const previous = `| 1 | ${test.child} | ${component} | ${beforeScore} | reactive-graph (${test.graph.nodes}) via template cyclomatic=1, reactive nodes=${test.graph.nodes} |`;
  const cycles = test.graph.cycles ? `, reactive cycles=${test.graph.cycles}` : "";
  const next = `| 1 | ${test.child} | ${component} | ${afterScore} | reactive-graph (${test.graph.score}) via template cyclomatic=1, reactive nodes=${test.graph.nodes}, reactive edges=${test.graph.edges}${cycles} |`;
  expected = replaceOnce(expected, previous, next);
  return expected;
}
