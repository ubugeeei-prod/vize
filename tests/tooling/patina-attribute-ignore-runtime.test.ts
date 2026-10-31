import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { projectViteConfigSource } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
import { directory, hash } from "./support/patina-attribute-ignore-contract.ts";

const bytes = fs.readFileSync(path.join(directory, "runtime-controls.original.json"));
assert.equal(hash(bytes), "96711d8ae2e3d73365ed768b3acff4334bf0d241cbf13764f4a95102a076a5fa");
const packet = JSON.parse(bytes.toString()) as {
  producer: { bytes: number; sha256: string; revision: string };
  rows: { id: string; input: unknown; outcome: unknown }[];
};
const original = fs.readFileSync(path.join(directory, "vite-runtime-6852.mjs.fixture"));
assert.equal(original.length, packet.producer.bytes);
assert.equal(hash(original), "3efc312a8430dac0e331dc823fd3fd134f59c694404b3283da0eb851b6663de0");
assert.equal(hash(original), packet.producer.sha256);
const legacy = (await import("data:text/javascript;base64," + original.toString("base64"))) as {
  projectViteConfigSource(value: unknown): Promise<unknown>;
};
async function outcome(project: (value: unknown) => Promise<unknown>, value: unknown) {
  try {
    return { kind: "returned", value: await project(structuredClone(value)) };
  } catch (error) {
    assert(error instanceof Error);
    return { kind: "threw", error: { name: error.name, message: error.message } };
  }
}

test("extracted runtime keeps sixteen whole original enum, union and scoped outcomes", async () => {
  assert.equal(packet.rows.length, 16);
  for (const row of packet.rows) {
    assert.deepEqual(await outcome(legacy.projectViteConfigSource, row.input), row.outcome, row.id);
    assert.deepEqual(await outcome(projectViteConfigSource, row.input), row.outcome, row.id);
  }
});
