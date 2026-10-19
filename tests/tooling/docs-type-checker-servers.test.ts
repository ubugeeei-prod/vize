import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";

const root = resolve(import.meta.dirname, "../..");
const read = (path: string) => readFileSync(resolve(root, path), "utf8");

test("documented typeChecker.servers range follows the native limit", () => {
  const native = read("crates/vize/src/commands/check/runner/resolve.rs");
  const maximum = native.match(/const MAX_CORSA_SERVERS: usize = (\d+);/)?.[1];
  assert.ok(maximum);
  assert.match(native, /if servers == 0/, "the documented minimum is 1");
  for (const locale of ["", "ja/"]) {
    const reference = read(`docs/content/${locale}guide/configuration-reference.md`);
    const line = reference.split("\n").find((row) => row.includes("`vue.typeChecker.servers`"));
    assert.ok(line, `${locale}configuration-reference documents servers`);
    assert.match(line, new RegExp(`\`1\`[–〜-]\`${maximum}\``), `${locale}: 1..${maximum}`);
  }
});
