import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const entry = fileURLToPath(
  new URL("../../tools/benchmarks/scripts/nuxt-bridge-transform.mjs", import.meta.url),
);

await test("the Nuxt bridge benchmark loads its real provider outside the checkout", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-nuxt-bridge-entry-"));
  try {
    const result = spawnSync(process.execPath, [entry, "--modules", "10", "--runs", "1"], {
      cwd,
      encoding: "utf8",
      env: { ...process.env, NODE_PATH: "" },
      timeout: 10_000,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stderr, "");
    const lines = result.stdout.trimEnd().split("\n");
    assert.deepEqual(lines.slice(0, 3), [
      "modules: 10, runs: 1",
      "| stage | gated | modules doing work | ms per pass | ms per module |",
      "| --- | --- | ---: | ---: | ---: |",
    ]);
    for (const [index, name] of [
      "1. component auto-imports",
      "2. i18n helper injection",
      "4. stable injected keys",
    ].entries()) {
      const cells = lines[index + 3].split(" | ");
      assert.deepEqual(cells.slice(0, 3), [`| ${name}`, "yes", "1 / 10"]);
      assert.match(cells[3], /^\d+\.\d{3}$/);
      assert.match(cells[4], /^\d+\.\d{6} \|$/);
    }
    assert.deepEqual(lines.slice(6), [
      "| 3. composable auto-imports | **no** | 10 / 10 | not measured | not measured |",
      "",
      "`unimport` does not resolve from this checkout, so stage 3 was not timed. It is Nuxt's",
      "dependency: run this from a Nuxt app's node_modules to time it. The counts above still hold —",
      "stage 3 has no fast-path gate, so it reaches every module while its three siblings skip most.",
    ]);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
