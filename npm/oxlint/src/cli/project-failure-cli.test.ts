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
      "../../../../tests/_fixtures/differential/lint/oxlint-script-safe-carrier-7903/project-failure.json",
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
      `
      const args = process.argv.slice(2);
      if (args.includes('--version')) { console.log('Version: 1.78.0'); process.exit(0); }
      if (args.includes('--debug')) { console.log(args.at(-1)); process.exit(0); }
      if (args.some(arg => arg.includes('oxlint-vize-original-'))) {
        process.stdout.write(${JSON.stringify(original)});
        process.stderr.write('whole original stderr\\n'); process.exit(${failure === "metadata" ? 1 : 2});
      }
      process.stdout.write(${JSON.stringify(["json", "metadata"].includes(failure) ? "{}\n" : "")});
      process.stderr.write('authored bridge failure\\n');
      process.exit(${failure === "json" ? 0 : failure === "metadata" ? 2 : 1});
    `,
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
