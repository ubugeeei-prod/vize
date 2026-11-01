import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  initializeCapabilities,
  INITIALIZE_CAPABILITIES,
  loadLspManifest,
} from "../differential/lsp-manifest.ts";
import { asArray, asRecord, parseJson } from "../../tools/support/json-value.ts";
import { runLspPack } from "../differential/lsp.ts";

const pack = new URL("../_fixtures/differential/lsp/", import.meta.url);
const loaded = loadLspManifest(fileURLToPath(new URL("manifest.json", pack)));
const id = "lsp/regression/versioned-template-lint-edits-6876";
const current = loaded.cases.find((row) => row.id === id);
assert(current);
const original = loaded.cases.find(
  (row) => row.id === "lsp/fix-history/code-action-utf16-original",
);
assert(original);
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const read = (path: string) => readFileSync(new URL(path, pack));

void test("versioned template profile preserves its mixed oracle and declares the whole suppression successor", () => {
  const oldSource = read("code-action-utf16-original/CodeActionUtf16.vue.txt");
  const oldResponse = read("code-action-utf16-original/response.expected.json");
  assert.equal(hash(oldSource), "4e334c5f8bdef19e5e77408a7aff23ef11f57362c5171401f6a5f5932c3fc1fd");
  assert.equal(
    hash(oldResponse),
    "6bdbc749bffaab95aad63b97553b99c115239210734d9c3c4d9bd8340890ef7a",
  );
  assert.deepEqual(read("versioned-template-lint-edits-6876/CodeActionUtf16.vue.txt"), oldSource);
  assert.deepEqual(current.data.request, original.data.request);
  assert.deepEqual(current.data.initializationOptions, original.data.initializationOptions);
  assert.equal(current.data.documentVersion, 1);
  assert.equal(current.data.workspaceEditDocumentChanges, true);
  const before = asArray(parseJson(oldResponse.toString("utf8")));
  const after = asArray(
    parseJson(read("versioned-template-lint-edits-6876/response.expected.json").toString("utf8")),
  );
  assert.equal(after.length, 3);
  const first = asRecord(before[0]);
  assert.deepEqual(after, [
    {
      ...first,
      edit: {
        documentChanges: [
          {
            textDocument: { uri: "${workspace}/CodeActionUtf16.vue", version: 1 },
            edits: asRecord(asRecord(first.edit).changes)["${workspace}/CodeActionUtf16.vue"],
          },
        ],
      },
    },
    ...before.slice(1),
  ]);
  assert.equal(
    hash(read("versioned-template-lint-edits-6876/case.json")),
    "6d94d5c8ce5a79261c1a1110a80d9e24387289c4c612346eb12e66a4143b85f9",
  );
  assert.equal(
    hash(read("versioned-template-lint-edits-6876/response.expected.json")),
    "3888a6a6fff7c9c9b7151957100f0f9e06e04e830439929f8c359565af62e24c",
  );
  const provenance = asRecord(current.data.provenance);
  assert.deepEqual(provenance.previousCase, {
    path: "case.json",
    sha256: "6d94d5c8ce5a79261c1a1110a80d9e24387289c4c612346eb12e66a4143b85f9",
  });
  assert.deepEqual(provenance.previousExpected, {
    path: "response.expected.json",
    sha256: "3888a6a6fff7c9c9b7151957100f0f9e06e04e830439929f8c359565af62e24c",
  });
  assert.equal(
    current.case.path,
    "versioned-template-lint-edits-6876/case.with-versioned-suppressions.json",
  );
  const successor = asArray(
    parseJson(
      read(
        "versioned-template-lint-edits-6876/response.with-versioned-suppressions.expected.json",
      ).toString("utf8"),
    ),
  );
  assert.deepEqual(successor, [
    after[0],
    ...after.slice(1).map((value) => {
      const action = asRecord(value);
      return {
        ...action,
        edit: {
          documentChanges: [
            {
              textDocument: { uri: "${workspace}/CodeActionUtf16.vue", version: 1 },
              edits: asRecord(asRecord(action.edit).changes)["${workspace}/CodeActionUtf16.vue"],
            },
          ],
        },
      };
    }),
  ]);
  assert.deepEqual(current.requests[0].result, successor);
});

void test("only the explicit client profile adds documentChanges without mutating existing capabilities", () => {
  const before = structuredClone(INITIALIZE_CAPABILITIES);
  assert.deepEqual(initializeCapabilities(original), before);
  assert.deepEqual(initializeCapabilities(current), {
    ...before,
    workspace: { workspaceEdit: { documentChanges: true } },
  });
  assert.deepEqual(INITIALIZE_CAPABILITIES, before);
  assert.deepEqual(initializeCapabilities(original), before);
  for (const fixture of loaded.cases) {
    if (fixture.data.workspaceEditDocumentChanges !== undefined) continue;
    const previousProfile =
      fixture.data.method === "textDocument/documentSymbol"
        ? {
            textDocument: {
              ...before.textDocument,
              documentSymbol: { hierarchicalDocumentSymbolSupport: true },
            },
          }
        : before;
    assert.deepEqual(initializeCapabilities(fixture), previousProfile, fixture.id);
  }
  assert.deepEqual(INITIALIZE_CAPABILITIES, before);
});

void test("unknown or misrouted capability metadata refuses instead of silently widening the profile", () => {
  const wrongValue = { ...current, data: { ...current.data } };
  Reflect.set(wrongValue.data, "workspaceEditDocumentChanges", false);
  assert.throws(
    () => initializeCapabilities(wrongValue),
    /explicit versioned edit client required/,
  );
  const wrongMethod = { ...current, data: { ...current.data, method: "textDocument/hover" } };
  assert.throws(
    () => initializeCapabilities(wrongMethod),
    /versioned edit fixture requires code actions/,
  );
  assert.equal(current.data.workspaceEditDocumentChanges, true);
  assert.equal(current.data.method, "textDocument/codeAction");
});

void test("source-built stdio preserves unadvertised originals and explicit versioned template successors", async (t) => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const revision = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(revision.status, 0, revision.stderr);
  const output = path.join(root, "target/differential/versioned-lint-actions-6876.json");
  const report = await runLspPack({
    manifestPath: fileURLToPath(new URL("manifest.json", pack)),
    binaryPath: path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize"),
    sourceRevision: revision.stdout.trim(),
    repoRoot: root,
    capture(observed) {
      mkdirSync(path.dirname(output), { recursive: true });
      writeFileSync(output, `${JSON.stringify(observed, null, 2)}\n`);
      // Preserve the whole record even when the existing validator refuses it.
      t.diagnostic(`versioned-lint-actions-whole-report ${JSON.stringify(observed)}`);
    },
  });
  const unsupported = loaded.cases.filter((fixture) => fixture.data.sourceSession).length;
  assert.deepEqual(report.summary, {
    plannedCases: 82,
    legacyMatches: 82 - unsupported,
    legacyFailures: 0,
    legacyUnsupported: unsupported,
    baselineDrift: 0,
    nativeUnsupported: 82,
    pairedComparisons: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
  });
});
