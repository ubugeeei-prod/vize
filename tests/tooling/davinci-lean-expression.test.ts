import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const table = JSON.parse(
  fs.readFileSync(path.join(repoRoot, "formal/impeto/fixtures/expression-cases.json"), "utf8"),
) as {
  context: Record<string, unknown>;
  cases: Array<{ expr: string; value?: unknown; unsupported?: string }>;
};

function evaluate(expr: string): unknown {
  // The oracle is the JavaScript engine itself, not a second hand-written evaluator.
  // Supported results are primitives, so values compare across the vm realm.
  return vm.runInNewContext(`"use strict"; (${expr});`, structuredClone(table.context));
}

test("TS-29 expression subset agrees with the JavaScript engine", () => {
  const supported = table.cases.filter((entry) => Object.hasOwn(entry, "value"));
  const unsupported = table.cases.filter((entry) => Object.hasOwn(entry, "unsupported"));
  assert.equal(supported.length + unsupported.length, table.cases.length);
  assert.ok(supported.length >= 50 && unsupported.length >= 30, "expression table is too small");
  for (const entry of supported) {
    assert.ok(!Object.hasOwn(entry, "unsupported"), `${entry.expr} has two verdicts`);
    assert.deepEqual(evaluate(entry.expr), entry.value, entry.expr);
  }
  for (const entry of unsupported) {
    assert.ok((entry.unsupported ?? "").length > 0, `${entry.expr} needs a reason`);
  }
});

test("TS-29 expression table covers each operator of the subset", () => {
  const exprs = table.cases.filter((entry) => Object.hasOwn(entry, "value")).map((e) => e.expr);
  for (const operator of ["*", "%", "+", "-", "<", "<=", ">=", ">", "===", "!==", "!"]) {
    assert.ok(
      exprs.some((expr) => expr.includes(operator)),
      `no case exercises ${operator}`,
    );
  }
  for (const operator of ["&&", "||", "?", ".length", "("]) {
    assert.ok(
      exprs.some((expr) => expr.includes(operator)),
      `no case exercises ${operator}`,
    );
  }
});
