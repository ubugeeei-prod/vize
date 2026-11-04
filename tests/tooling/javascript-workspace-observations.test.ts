import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";
import test from "node:test";
import { currentNoDupeKeysPacket } from "../../tools/support/compat/nuxt/javascript-workspace-lint.ts";
import { ownedSessionDirectory } from "./support/lsp/javascript-workspace-captures.ts";

const historical = [
  {
    file: "lint-options.js",
    messages: [
      {
        ruleId: "script/no-dupe-keys",
        ruleDocsPath: "docs/content/rules/type-and-script.md",
        severity: 2,
        message: "[vize:script/no-dupe-keys] Duplicated key 'value'",
        line: 4,
        column: 14,
        endLine: 4,
        endColumn: 19,
        help: "retain the complete original help",
      },
    ],
    errorCount: 1,
    warningCount: 0,
  },
];

void test("the explicit docs move preserves the whole historical lint packet", () => {
  const before = JSON.stringify(historical);
  const current = structuredClone(historical);
  current[0].messages[0].ruleDocsPath = "docs/content/rules/reference/script-no-dupe-keys.md";
  assert.deepEqual(currentNoDupeKeysPacket(historical), current);
  assert.equal(JSON.stringify(historical), before);
});

void test("the docs transition refuses a different rule, source URL or cardinality", () => {
  for (const field of ["ruleId", "ruleDocsPath"] as const) {
    const changed = structuredClone(historical);
    changed[0].messages[0][field] = "foreign";
    assert.throws(() => currentNoDupeKeysPacket(changed));
  }
  assert.throws(() => currentNoDupeKeysPacket([]));
  const changed = structuredClone(historical);
  changed[0].messages.push({ ...changed[0].messages[0] });
  assert.throws(() => currentNoDupeKeysPacket(changed));
});

void test("parallel capture producers each retain exactly their own session", async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-js-workspace-sessions-"));
  try {
    const before = fs.readdirSync(directory);
    const produce = promisify(execFile);
    const results = await Promise.all(
      [0, 1, 2].map(() =>
        produce(
          process.execPath,
          [
            "--input-type=module",
            "-e",
            'import fs from "node:fs"; import path from "node:path"; fs.mkdtempSync(path.join(process.argv[1], `${process.pid}-1-`)); console.log(process.pid);',
            directory,
          ],
          { timeout: 5000 },
        ),
      ),
    );
    const after = fs.readdirSync(directory);
    assert.equal(after.length, 3);
    for (const result of results) {
      const pid = Number(result.stdout.trim());
      assert.ok(ownedSessionDirectory(before, after, pid).startsWith(`${pid}-`));
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("session ownership rejects missing and duplicate own captures", () => {
  const before = ["12-1-old"];
  assert.equal(ownedSessionDirectory(before, [...before, "12-2-new"], 12), "12-2-new");
  assert.throws(() => ownedSessionDirectory(before, [...before, "123-2-foreign"], 12));
  assert.throws(() => ownedSessionDirectory(before, [...before, "12-2-a", "12-3-b"], 12));
  assert.throws(() => ownedSessionDirectory([], [], 0));
});
