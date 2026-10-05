import assert from "node:assert/strict";
import path from "node:path";
import { loadProductManifest, readPinnedArtifact } from "./harness.mjs";
import { loadOriginalHighlightRequests } from "./lsp-highlight-manifest.ts";
import type { FixtureData, LoadedLspManifest, PlannedLspFixture, LspFixture } from "./lsp-types.ts";

export const NATIVE_REASON = "native whole-product LSP adapter unavailable";

export function safeRuntimePath(value: unknown): string {
  assert(typeof value === "string");
  assert(value.length > 0 && !path.isAbsolute(value), "relative runtime path required");
  assert(!value.includes("\\"), "runtime paths use slash separators");
  assert(value.split("/").every((part) => part !== "" && part !== "." && part !== ".."));
  return value;
}

export function loadLspManifest(manifestPath: string): LoadedLspManifest {
  const loaded = loadProductManifest(manifestPath, "lsp");
  const packRoot = path.dirname(manifestPath);
  loaded.cases = (loaded.cases as PlannedLspFixture[]).map((planned): LspFixture => {
    assert.deepEqual(planned.targets, ["stdio"]);
    assert(Number.isSafeInteger(planned.requestCount) && planned.requestCount > 0);
    const data = JSON.parse(
      readPinnedArtifact(packRoot, planned.case).toString("utf8"),
    ) as FixtureData;
    assert.equal(data.version, 1);
    assert.equal(data.id, planned.id);
    assert.match(data.provenance.fixCommit, /^[a-f0-9]{40}$/);
    assert.equal(data.documentVersion, 1);
    assert.deepEqual(planned.initializationOptions, {
      editor: true,
      lint: false,
      typecheck: false,
      ...(data.method === "textDocument/onTypeFormatting" ? { formatting: true } : {}),
    });
    if (data.initializationOptions) {
      assert.deepEqual(planned.initializationOptions, data.initializationOptions);
    }
    const fixtureRoot = path.resolve(packRoot, path.dirname(planned.case.path));
    assert.equal(planned.inputs.root, path.dirname(planned.case.path));
    const rawFiles = data.files ?? (data.input ? [data.input] : []);
    assert(rawFiles.length > 0, "immutable input files are required");
    assert.deepEqual(
      planned.inputs.files,
      rawFiles.map((file) => ({
        path: file.source,
        sha256: file.sha256,
      })),
    );
    const files = rawFiles.map((file) => ({
      runtimePath: safeRuntimePath(file.runtimePath),
      bytes: readPinnedArtifact(fixtureRoot, { path: file.source, sha256: file.sha256 }),
    }));
    assert.equal(new Set(files.map((file) => file.runtimePath)).size, files.length);
    const entry = safeRuntimePath(data.entry ?? data.input?.runtimePath);
    assert(
      files.some((file) => file.runtimePath === entry),
      "entry must be an immutable input",
    );
    const expected = JSON.parse(
      readPinnedArtifact(fixtureRoot, {
        path: data.expected.source,
        sha256: data.expected.sha256,
      }).toString("utf8"),
    ) as unknown;
    let requests: LspFixture["requests"];
    if (data.method === "textDocument/documentLink") {
      assert(
        Array.isArray(expected) && expected.length === 3,
        "three original active links required",
      );
      requests = [{ params: {}, result: expected }];
    } else if (data.method === "textDocument/foldingRange") {
      assert(expected === null || (Array.isArray(expected) && expected.length > 0));
      requests = [{ params: {}, result: expected }];
    } else if (data.method === "textDocument/documentHighlight") {
      requests = loadOriginalHighlightRequests(packRoot, data, expected);
    } else if (data.method === "textDocument/documentSymbol") {
      assert.equal(data.hierarchicalDocumentSymbols, true);
      assert(Array.isArray(expected) && expected.length === 2);
      assert(data.provenance.witness, "original issue custody is required");
      readPinnedArtifact(packRoot, data.provenance.witness);
      requests = [{ params: {}, result: expected }];
    } else {
      assert.equal(data.method, "textDocument/onTypeFormatting", "unregistered LSP method");
      assert.deepEqual(data.options, { tabSize: 2, insertSpaces: true });
      assert(Array.isArray(expected) && expected.length === 4, "all four CRLF controls required");
      requests = (expected as { position: unknown; ch: string; result: unknown }[]).map(
        ({ position, ch, result }) => ({
          params: { position, ch, options: data.options },
          result,
        }),
      );
    }
    assert.equal(requests.length, planned.requestCount, "all authored requests must be planned");
    return { ...planned, data, files, entry, requests };
  });
  return loaded as LoadedLspManifest;
}

export function materializeWorkspace(value: unknown, workspaceUri: string): unknown {
  if (typeof value === "string") return value.replaceAll("${workspace}", workspaceUri);
  if (Array.isArray(value)) return value.map((item) => materializeWorkspace(item, workspaceUri));
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, materializeWorkspace(item, workspaceUri)]),
    );
  }
  return value;
}

export const INITIALIZE_CAPABILITIES = {
  textDocument: {
    completion: { completionItem: { documentationFormat: ["markdown", "plaintext"] } },
  },
};

export function initializeCapabilities(fixture: LspFixture) {
  if (fixture.data.method !== "textDocument/documentSymbol") return INITIALIZE_CAPABILITIES;
  assert.equal(fixture.data.hierarchicalDocumentSymbols, true);
  return {
    textDocument: {
      ...INITIALIZE_CAPABILITIES.textDocument,
      documentSymbol: { hierarchicalDocumentSymbolSupport: true },
    },
  };
}
