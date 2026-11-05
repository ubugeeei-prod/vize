import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { emitGateOracle } from "./support/emit-props-gate-oracle.ts";

const repo = new URL("../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, repo), "utf8");
const oracle = emitGateOracle(read);

void test("the current resolver source successor preserves frozen proof and refuses changed source, closure or repins", () => {
  const fixture = "tests/_fixtures/tooling/emit-props-gate-current-source-20261105.json";
  const successor = JSON.parse(read(fixture));
  assert.deepEqual(oracle, JSON.parse(read(successor.predecessor.path)));
  for (const path of [
    fixture,
    ...successor.dependencies.map((entry: { path: string }) => entry.path),
  ]) {
    for (const body of ["", read(path) + "\n"])
      assert.throws(() =>
        emitGateOracle((selected) => (selected === path ? body : read(selected))),
      );
  }
  for (const entry of successor.sources) {
    const source = read(entry.path);
    const hunk = entry.hunks[0];
    for (const changed of [
      `// unrelated prefix\n${source}`,
      source + "\n",
      source.replace(hunk.after, ""),
      source.replace(hunk.after, hunk.after + hunk.after),
      source.replace(hunk.after, hunk.after.replace("\n", "\n// unregistered delta\n")),
    ]) {
      assert.notEqual(changed, source);
      assert.throws(
        () => emitGateOracle((path) => (path === entry.path ? changed : read(path))),
        /selected emit gate producer/u,
      );
      const repinned = structuredClone(successor);
      repinned.sources.find((source: { path: string }) => source.path === entry.path).after = {
        bytes: Buffer.byteLength(changed),
        sha256: createHash("sha256").update(changed).digest("hex"),
      };
      assert.throws(
        () =>
          emitGateOracle((path) =>
            path === fixture
              ? JSON.stringify(repinned)
              : path === entry.path
                ? changed
                : read(path),
          ),
        /selected emit gate producer/u,
      );
    }
  }
});
