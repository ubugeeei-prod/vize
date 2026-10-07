import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { prepareScopedSelection } from "./scoped-selection.ts";
import { unavailableProjectTransport } from "./project-output.ts";

const fixture = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../../../tests/_fixtures/differential/lint/oxlint-script-safe-carrier-7903/project-failure.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as { original: Record<string, unknown> };
const originalPacket = {
  stdout: JSON.stringify(fixture.original) + "\n",
  stderr: "whole original stderr\n",
  status: 1,
};

for (const mode of ["mirror", "selection", "spawn"] as const)
  void test(`a ${mode} failure after original linting retains raw fields and duplicate fatals`, async (t) => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-project-failure-"));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    const file = path.join(root, "Original.vue");
    const bytes = '<template><div v-html="html" /></template>\n';
    fs.writeFileSync(file, bytes);
    const config = JSON.stringify({
      rules: { "vize/vue/no-v-html": "error" },
      ...(mode === "mirror" ? { overrides: [{ files: ["!**/*.vue"], rules: {} }] } : {}),
    });
    fs.writeFileSync(path.join(root, ".oxlintrc.json"), config);
    let selections = 0;
    const result = await prepareScopedSelection(
      root,
      ["-f", "json", "."],
      [file],
      async (args) => {
        if (args.includes("--debug")) {
          selections++;
          if (selections <= 2) return { status: 0, stdout: "Original.vue\n", stderr: "" };
          if (mode === "spawn") throw new Error("authored bridge spawn failure");
          return {
            status: 2,
            stdout: "authored bridge selection failure\n",
            stderr: "whole selection stderr\n",
          };
        }
        return originalPacket;
      },
      new Set([file]),
    );
    assert.ok(result && "result" in result);
    assert.equal(result.result.stdout, originalPacket.stdout);
    assert.deepEqual(JSON.parse(result.result.stdout), fixture.original);
    assert.ok(result.result.stderr.startsWith(originalPacket.stderr));
    assert.match(result.result.stderr, /Script-safe Vue transport unavailable/u);
    assert.ok(result.result.status != null && result.result.status >= 1);
    assert.equal(fs.readFileSync(file, "utf8"), bytes);
    assert.equal(fs.readFileSync(path.join(root, ".oxlintrc.json"), "utf8"), config);
    assert.deepEqual(fs.readdirSync(root).sort(), [".oxlintrc.json", "Original.vue"]);
  });

void test("transport failure promotes only otherwise-successful original reports", () => {
  for (const status of [0, 1, 2, null]) {
    const original = { ...originalPacket, status };
    const failed = unavailableProjectTransport(original, new Error("authored unavailable"));
    assert.equal(failed.stdout, original.stdout);
    assert.equal(failed.status, status === 0 || status == null ? 1 : status);
    assert.equal(
      failed.stderr,
      original.stderr + "\nScript-safe Vue transport unavailable: authored unavailable\n",
    );
  }
});
