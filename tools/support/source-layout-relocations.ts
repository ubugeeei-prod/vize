import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

// Historical artifacts keep their original paths and hashes. These explicit
// coordinates locate their live owners after the ordinary-module migration.
const inventory: { version: number; issue: number; from: string; paths: Record<string, string> } =
  JSON.parse(
    readFileSync(
      new URL("../../tests/_fixtures/source-layout/10418.json", import.meta.url),
      "utf8",
    ),
  );
assert.equal(inventory.version, 1);
assert.equal(inventory.issue, 10418);
assert.equal(inventory.from, "3146e4fa530b8b886d8429ae379096bfcc791cad");
const forward = new Map(Object.entries(inventory.paths));
const reverse = new Map([...forward].map(([before, after]) => [after, before]));
assert.equal(reverse.size, forward.size, "source relocation must be unambiguous");
for (const [before, after] of forward) {
  for (const value of [before, after]) {
    assert.match(value, /^(?:crates|davinci|editors|tests|tools|vendor)\//);
    assert(!value.split("/").some((part) => part === ".." || part === "."));
    assert(!value.includes("\0"));
  }
}

export function currentSourcePath(original: string): string {
  return forward.get(original) ?? original;
}

export function previousSourcePath(current: string): string {
  return reverse.get(current) ?? current;
}
