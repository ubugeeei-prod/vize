import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "vite-plus/test";

const require = createRequire(import.meta.url);
const native = require("@vizejs/native") as { normalizeVizeConfig(value: unknown): unknown };

const TOO_DEEP = /config objects and arrays may nest at most 128 levels/u;

/** A config whose `entries` array starts a chain of `levels` nested objects. */
function nestedEntries(levels: number): unknown {
  let value: Record<string, unknown> = {};
  for (let level = 1; level < levels; level += 1) value = { next: value };
  return { entries: [value] };
}

test("config objects nested up to the bound normalize as before", () => {
  // The root object and `entries` array are two of the 128 levels.
  const resolved = native.normalizeVizeConfig(nestedEntries(126)) as { entries: unknown[] };
  let depth = 0;
  let value = resolved.entries[0] as { next?: unknown } | undefined;
  while (value !== undefined) {
    depth += 1;
    value = value.next as { next?: unknown } | undefined;
  }
  assert.equal(depth, 126);
});

test("config objects nested past the bound are reported, not walked", () => {
  assert.throws(() => native.normalizeVizeConfig(nestedEntries(127)), TOO_DEEP);
  assert.throws(() => native.normalizeVizeConfig(nestedEntries(100_000)), TOO_DEEP);
  const nested: unknown[] = [];
  let array = nested;
  for (let level = 0; level < 100_000; level += 1) {
    const next: unknown[] = [];
    array.push(next);
    array = next;
  }
  assert.throws(() => native.normalizeVizeConfig({ vue: { plugins: nested } }), TOO_DEEP);
});

test("a self-referencing config object is reported", () => {
  const linter: Record<string, unknown> = { rules: {} };
  linter.self = linter;
  assert.throws(() => native.normalizeVizeConfig({ linter }), TOO_DEEP);
  const entry: Record<string, unknown> = { linter: {} };
  (entry.linter as Record<string, unknown>).loop = entry;
  assert.throws(() => native.normalizeVizeConfig([entry]), TOO_DEEP);
});
