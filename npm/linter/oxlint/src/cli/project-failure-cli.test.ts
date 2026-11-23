import { readToolingProgram } from "../../../../../tests/tooling/support/program-path.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const cli = fileURLToPath(new URL("../../dist/cli.mjs", import.meta.url));
const fixture = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../../../../tests/_fixtures/differential/lint/oxlint-script-safe-carrier-7903/project-failure.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as { original: Record<string, unknown> };

for (const failure of ["preparation", "json", "unavailable", "metadata"])
  void test(`the packaged CLI retains original status 2 and raw packets after ${failure} failure`, (t) => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-project-cli-failure-"));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    const original = JSON.stringify(fixture.original) + "\n";
    const bytes = '<template><div v-html="html" /></template>\n';
    fs.writeFileSync(path.join(root, "Original.vue"), bytes);
    const config = JSON.stringify({
      rules: { "vize/vue/no-v-html": "error" },
      ...(failure === "preparation" ? { overrides: [{ files: ["!**/*.vue"], rules: {} }] } : {}),
    });
    fs.writeFileSync(path.join(root, ".oxlintrc.json"), config);
    const engine = path.join(root, "node_modules/oxlint/bin/oxlint");
    fs.mkdirSync(path.dirname(engine), { recursive: true });
    // Authored protocol process, not a real-host or source-native oracle.
    fs.writeFileSync(
      engine,
      readToolingProgram("commands/project-failure-cli/emit-engine-failure.js.in", {
        ORIGINAL_STDOUT: JSON.stringify(original),
        METADATA_STATUS: failure === "metadata" ? 1 : 2,
        BRIDGE_STDOUT: JSON.stringify(["json", "metadata"].includes(failure) ? "{}\n" : ""),
        BRIDGE_STATUS: failure === "json" ? 0 : failure === "metadata" ? 2 : 1,
      }),
    );
    const result = spawnSync(process.execPath, [cli, "-f", "json", "Original.vue"], {
      cwd: root,
      encoding: "utf8",
      timeout: 10_000,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 2, result.stderr);
    assert.equal(result.stdout, original);
    assert.ok(result.stderr.startsWith("whole original stderr\n"));
    assert.equal(fs.readFileSync(path.join(root, "Original.vue"), "utf8"), bytes);
    assert.equal(fs.readFileSync(path.join(root, ".oxlintrc.json"), "utf8"), config);
    assert.deepEqual(fs.readdirSync(root).sort(), [
      ".oxlintrc.json",
      "Original.vue",
      "node_modules",
    ]);
  });
