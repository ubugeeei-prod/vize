import assert from "node:assert/strict";
import { stripTypeScriptTypes } from "node:module";
import vm from "node:vm";
import { stripRust } from "../../tools/support/compat/davinci/lib/rust-source.mjs";

// These source witnesses qualify original inputs only; they never execute a
// formatter or manufacture result/error/runtime credit.
export function nativeFunction(bytes, name) {
  const raw = bytes.toString();
  assert(Buffer.from(raw).equals(bytes));
  const clean = stripRust(raw);
  const pattern = new RegExp(`\\bfn ${name}\\(`, "g");
  const found = pattern.exec(clean);
  assert(found);
  assert.equal(pattern.exec(clean), null);
  const open = clean.indexOf("{", found.index);
  assert(open >= 0);
  let depth = 1;
  let end = open + 1;
  while (depth && end < clean.length) {
    if (clean[end] === "{") depth++;
    if (clean[end] === "}") depth--;
    end++;
  }
  assert.equal(depth, 0);
  return Buffer.from(raw.slice(found.index, end));
}

export function originalViteSortingControls(bytes, fixture) {
  const source = stripTypeScriptTypes(bytes.toString(), { mode: "strip" });
  const begin = source.indexOf("const controls");
  assert(begin >= 0 && source.indexOf("const controls", begin + 1) < 0);
  const open = source.indexOf("= [", begin) + 2;
  const end = source.indexOf("\n  ];", open) + 4;
  assert(open > begin && end > open);
  const original = vm.runInNewContext(source.slice(open, end), { fixture }, { timeout: 1_000 });
  assert.equal(original.length, 5);
  for (const control of original) {
    assert(Array.isArray(control) && control.length === 2);
    assert.equal(typeof control[0], "string");
    assert(control[1] && typeof control[1] === "object");
  }
  return JSON.parse(JSON.stringify(original));
}
