import assert from "node:assert/strict";
import { linuxCpuAffinity } from "./cpu-affinity.ts";
const affinity = linuxCpuAffinity();
const [mode, argument, ...variables] = process.argv.slice(2);
if (mode === "first") {
  const count = Number(argument);
  assert.ok(Number.isInteger(count) && count > 0);
  console.log(affinity.slice(0, count).join(","));
} else {
  assert.equal(mode, "report");
  assert.ok(argument && variables.length);
  const runner = Object.fromEntries(variables.map((name) => [name, process.env[name] ?? null]));
  console.log(JSON.stringify({ runner, affinity, scope: argument }));
}
