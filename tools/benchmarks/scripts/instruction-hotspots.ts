import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { parseCallgrind } from "./instruction-counts-lib.mjs";

// Diagnostic-only reader for the existing single-part, Ir-only client dumps.
// Calls carry inclusive edge costs; ordinary rows carry exclusive self costs:
// https://valgrind.org/docs/manual/cl-format.html#cl-format.associations
export function parseHotspots(text: string) {
  const dump = parseCallgrind(text);
  assert.ok(dump, "hotspots require a named client dump");
  const functions = new Map<string, string>();
  const objects = new Map<string, string>();
  const rows = new Map<
    string,
    {
      object: string;
      function: string;
      exclusive: number;
      inclusive: number;
    }
  >();
  const resolve = (value: string, names: Map<string, string>) => {
    const compressed = value.match(/^\((\d+)\)(?:\s+(.*))?$/u);
    if (!compressed) return value;
    const [, id, name] = compressed;
    if (name !== undefined) {
      assert.ok(!names.has(id) || names.get(id) === name, "conflicting compressed name");
      names.set(id, name);
    }
    assert.ok(names.has(id), `unresolved compressed name ${id}`);
    return names.get(id)!;
  };
  const counter = (value: string) => {
    assert.match(value, /^(?:\d+|0x[\da-fA-F]+)$/u, "invalid Ir counter");
    const result = Number(value);
    assert.ok(Number.isSafeInteger(result) && result >= 0, "unsafe Ir counter");
    return result;
  };
  const add = (left: number, right: number) => {
    const result = left + right;
    assert.ok(Number.isSafeInteger(result), "unsafe accumulated Ir counter");
    return result;
  };
  let object = "(unknown object)";
  let fn: string | undefined;
  let positions = 1;
  let callCost = false;
  let hasCallee = false;
  let exclusiveTotal = 0;
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    if (/^(?:\*|[+-]?(?:\d|0x))/u.test(line)) {
      assert.ok(fn !== undefined, "cost row without an owning function");
      const fields = line.split(/\s+/u);
      assert.ok(
        fields.length >= positions && fields.length <= positions + 1,
        "unexpected position or event count",
      );
      for (const position of fields.slice(0, positions)) {
        assert.match(
          position,
          /^(?:\*|[+-]?(?:\d+|0x[\da-fA-F]+))$/u,
          "invalid compressed position",
        );
      }
      const cost = fields.length === positions ? 0 : counter(fields[positions]);
      const key = JSON.stringify([object, fn]);
      const row = rows.get(key) ?? { object, function: fn, exclusive: 0, inclusive: 0 };
      row.inclusive = add(row.inclusive, cost);
      if (!callCost) {
        row.exclusive = add(row.exclusive, cost);
        exclusiveTotal = add(exclusiveTotal, cost);
      }
      rows.set(key, row);
      callCost = false;
      continue;
    }
    assert.ok(!callCost, "call association is missing its cost row");
    const name = line.match(/^(ob|cob|fn|cfn)=(.*)$/u);
    if (name) {
      const value = resolve(name[2].trim(), name[1].endsWith("fn") ? functions : objects);
      if (name[1] === "ob") object = value;
      if (name[1] === "fn") fn = value;
      if (name[1] === "cfn") hasCallee = true;
    } else if (line.startsWith("calls=")) {
      assert.ok(hasCallee, "call association without a target function");
      assert.match(line, /^calls=(?:\d+|0x[\da-fA-F]+)\s+.+$/u, "invalid call association");
      callCost = true;
    } else if (line.startsWith("positions:")) {
      const names = line.slice("positions:".length).trim().split(/\s+/u);
      assert.ok(
        names.length >= 1 &&
          names.length <= 3 &&
          names.every((name) => ["instr", "bb", "line"].includes(name)),
        "unsupported positions",
      );
      assert.deepEqual(
        names,
        ["instr", "bb", "line"].filter((name) => names.includes(name)),
        "duplicate or reordered positions",
      );
      positions = names.length;
    } else {
      assert.match(
        line,
        /^(?:[\w ]+:|(?:fl|fi|fe|cfi|cfl|jump|jcnd)=)/u,
        "unsupported Callgrind row",
      );
    }
  }
  assert.ok(!callCost, "unterminated call association");
  assert.equal(exclusiveTotal, dump.instructions, "self costs disagree with measured total");
  return { ...dump, rows: [...rows.values()] };
}

export function reportOverBudgetHotspots(
  measurementFile: string,
  report: { runs: Array<Record<string, { instructions: number }>> },
  budget: { instruction: Record<string, { instructions: number }> },
  log: (line: string) => void = console.log,
) {
  const failed = new Set(
    Object.entries(report.runs[0])
      .filter(([id, row]) => row.instructions > budget.instruction[id].instructions)
      .map(([id]) => id),
  );
  if (!failed.size) return;
  // Read the already-completed first run; never rebuild or execute a workload.
  const directory = path.join(path.dirname(measurementFile), "run-1");
  const found = new Set<string>();
  for (const file of fs
    .readdirSync(directory)
    .filter((file) => /\.callgrind(?:\.\d+)?$/u.test(file))) {
    const text = fs.readFileSync(path.join(directory, file), "utf8");
    const trigger = text.match(/^desc: Trigger:\s*Client Request: (.+)$/mu)?.[1];
    if (!trigger || !failed.has(trigger)) continue;
    const dump = parseHotspots(text);
    assert.ok(!found.has(dump.bench_id), "duplicate hotspot dump");
    found.add(dump.bench_id);
    assert.equal(
      dump.instructions,
      report.runs[0][dump.bench_id].instructions,
      "hotspot dump does not match measured workload",
    );
    log(
      `instruction-hotspots: ${JSON.stringify({
        bench_id: dump.bench_id,
        instructions: dump.instructions,
        ceiling: budget.instruction[dump.bench_id].instructions,
      })}`,
    );
    for (const mode of ["exclusive", "inclusive"] as const) {
      const rows = dump.rows
        .filter((row) => row[mode] > 0)
        .sort(
          (a, b) =>
            b[mode] - a[mode] ||
            a.function.localeCompare(b.function) ||
            a.object.localeCompare(b.object),
        )
        .slice(0, 20);
      for (const row of rows) {
        log(
          `instruction-hotspot: ${JSON.stringify({
            bench_id: dump.bench_id,
            mode,
            instructions: row[mode],
            function: row.function,
            object: row.object,
          })}`,
        );
      }
    }
  }
  assert.deepEqual([...found].sort(), [...failed].sort(), "missing over-budget workload dump");
}
