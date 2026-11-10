// #6858: mounted parity of Vize's TSRX DOM output against the pinned reference
// pipeline. Each row mounts one component of a module under
// tests/_fixtures/tsrx/mount/ in both pipelines and compares the normalized
// DOM, element identities and runtime warnings after every step. The Vize side
// is the committed `<module>.dom.ts`, which `davinci/vize_l4/tests/tsrx_dom.rs`
// pins to the compiler's output.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import { type MountRow, mountRow } from "./jsx-vapor-oracle/tsrx-mount.ts";
import { repoRoot } from "./tsrx-oracle/reference.ts";

const mountRoot = join(repoRoot, "tests", "_fixtures", "tsrx", "mount");
const { rows } = JSON.parse(readFileSync(join(mountRoot, "mount.json"), "utf8")) as {
  rows: MountRow[];
};

for (const row of rows) {
  test(`mounted parity: ${row.id}`, async () => {
    const vize = readFileSync(join(repoRoot, row.file.replace(/\.tsrx$/u, ".dom.ts")), "utf8");
    const { reference, vize: observed } = await mountRow(row, vize);
    assert.ok(reference.length > row.steps.length, `${row.id}: every phase observed`);
    assert.deepEqual(observed, reference);
  });
}
