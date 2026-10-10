import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { offsetToPosition } from "./support/lsp/assertions.ts";
import { EDITOR_BUNDLE_CAPABILITIES } from "./support/lsp/capability-oracles.ts";
import {
  FRESH_EDITOR_CAPABILITIES,
  readPolicyFixture,
} from "./support/lsp/fresh-capability-policy.ts";

test("fresh LSP policy preserves the original whole tests, inputs and complete capability contract", () => {
  const custodyBytes = readPolicyFixture("custody.json");
  assert.equal(
    createHash("sha256").update(custodyBytes).digest("hex"),
    "bd2290e91b205837170cc3edb185f459d665267b32189089d7b457d313edba33",
  );
  const custody = JSON.parse(custodyBytes) as { files: Record<string, string> };
  for (const [name, expectedHash] of Object.entries(custody.files)) {
    assert.equal(createHash("sha256").update(readPolicyFixture(name)).digest("hex"), expectedHash);
  }
  assert.deepEqual(
    EDITOR_BUNDLE_CAPABILITIES,
    JSON.parse(readPolicyFixture("capabilities.historical.json")),
  );
  assert.deepEqual(FRESH_EDITOR_CAPABILITIES, {
    ...EDITOR_BUNDLE_CAPABILITIES,
    documentFormattingProvider: true,
    documentRangeFormattingProvider: true,
    documentOnTypeFormattingProvider: {
      firstTriggerCharacter: ";",
      moreTriggerCharacter: ["}", "\n"],
    },
    experimental: { vize: { jsxTypecheck: true } },
  });
  const broken = readPolicyFixture("Consumer.invalid.tsx.txt");
  const start = offsetToPosition(broken, broken.indexOf("count="));
  assert.deepEqual(JSON.parse(readPolicyFixture("jsx-diagnostics.authored.json")), {
    invalid: [
      {
        code: 2322,
        message: "Type 'string' is not assignable to type 'number'.",
        range: { start, end: { line: start.line, character: start.character + "count".length } },
        severity: 1,
        source: "vize/types",
      },
    ],
    valid: [],
    intrinsic: [],
  });
});
