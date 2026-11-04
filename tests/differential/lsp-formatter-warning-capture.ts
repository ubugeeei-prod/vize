import assert from "node:assert/strict";
import { readPinnedArtifact, sha256 } from "./harness.mjs";
import {
  FORMATTER_BIND_POLICY_CONFIG,
  FORMATTER_BIND_POLICY_FIXED,
} from "./lsp-formatter-bind-policy.ts";
import type { FixtureData, LspFixture } from "./lsp-types.ts";

export const FORMATTER_WARNING_CAPTURE_ID = "lsp/regression/formatter-warning-capture-11815";
export const FORMATTER_WARNING_CONSUMER =
  "crates/vize_maestro/src/server/handlers/tests/formatting/bind_style_policy.rs";
const TEST = "every_handler_logs_the_conflict_returns_null_and_keeps_authored_bytes";

/** Authenticate the new warning-capture input without reclassifying the old formatter law. */
export function loadFormatterWarningCapture(
  repositoryRoot: string,
  data: FixtureData,
  files: { runtimePath: string; bytes: Buffer }[],
  expected: unknown,
): LspFixture["requests"] {
  assert.equal(data.version, 1);
  assert.equal(data.id, FORMATTER_WARNING_CAPTURE_ID);
  assert.equal(data.provenance.fixCommit, "b0beefba53e3c1473d9bf4d7179a5b29b904ca96");
  assert.equal(data.method, "textDocument/formatting");
  assert.equal(data.documentVersion, 1);
  assert.equal(data.openDocument, undefined);
  assert.equal(data.sourceSession, undefined);
  assert.equal(data.request, undefined);
  assert.equal(data.entry, "VBindProp.vue");
  assert.equal(data.expected.source, "response.expected.json");
  assert.deepEqual(data.initializationOptions, {
    editor: true,
    lint: false,
    typecheck: false,
    formatting: true,
  });
  assert.deepEqual(
    files.map((file) => file.runtimePath),
    ["VBindProp.vue", "vite.config.mjs", "warnings.expected.json"],
  );
  assert.deepEqual(
    data.files?.map(({ source, runtimePath, sha256: digest }) => ({
      source,
      runtimePath,
      sha256: digest,
    })),
    files.map((file, index) => ({
      source: ["Fixed.vue.txt", "vite.config.mjs.txt", "warnings.expected.json"][index],
      runtimePath: file.runtimePath,
      sha256: sha256(file.bytes),
    })),
  );
  assert.equal(files[0]!.bytes.toString("utf8"), FORMATTER_BIND_POLICY_FIXED);
  assert.equal(files[1]!.bytes.toString("utf8"), FORMATTER_BIND_POLICY_CONFIG);
  const message =
    "Refusing formatter request for ${file}: normalizeDirectiveShorthands=true conflicts with active vue/v-bind-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand binding style";
  assert.deepEqual(
    JSON.parse(files[2]!.bytes.toString("utf8")),
    Array.from({ length: 3 }, () => ({ level: "WARN", fields: [["message", message]] })),
  );
  const options = { tabSize: 2, insertSpaces: true };
  assert.deepEqual(data.options, options);
  assert.deepEqual(expected, [
    { method: "textDocument/formatting", params: { options }, result: null },
    {
      method: "textDocument/rangeFormatting",
      params: {
        range: { start: { line: 0, character: 12 }, end: { line: 0, character: 40 } },
        options,
      },
      result: null,
    },
    {
      method: "textDocument/onTypeFormatting",
      params: { position: { line: 0, character: 24 }, ch: "\n", options },
      result: null,
    },
  ]);
  assert.equal(data.provenance.witness?.path, FORMATTER_WARNING_CONSUMER);
  const witness = data.provenance.witness!;
  const source = readPinnedArtifact(repositoryRoot, witness).toString("utf8");
  const start = source.indexOf(`fn ${TEST}()`);
  assert(start >= 0, "the existing actual handler test must consume the new input");
  const body = source.slice(start, source.indexOf("\n#[test]", start));
  for (const input of [
    "Fixed.vue.txt",
    "vite.config.mjs.txt",
    "response.expected.json",
    "warnings.expected.json",
  ]) {
    assert(body.includes(`/formatter-warning-capture-11815/${input}`), `unused input: ${input}`);
  }
  for (const law of [
    "assert_eq!(source, original_fixed());",
    "assert_eq!(result, [None, None, None]);",
    "serde_json::to_value(&result).unwrap()",
    "assert_eq!(logs, vec![expected; 3]);",
    "assert_eq!(logs, expected_logs);",
    "rebuild_interest_from_unsubscribed_thread();",
  ]) {
    assert(body.includes(law), `missing actual consumer law: ${law}`);
  }
  return expected as LspFixture["requests"];
}
