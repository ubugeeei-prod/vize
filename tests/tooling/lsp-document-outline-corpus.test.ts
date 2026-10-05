import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { initializeCapabilities, INITIALIZE_CAPABILITIES } from "../differential/lsp-manifest.ts";
import { loadLspManifest } from "../differential/lsp.ts";
import { inspectLspObservation } from "../differential/lsp-report.ts";
import type { JsonRpcMessage, LspFixture } from "../differential/lsp-types.ts";
import { referenceObservation, rewriteFrames } from "./support/lsp/differential-reference.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const loaded = loadLspManifest(path.join(root, "tests/_fixtures/differential/lsp/manifest.json"));
const originals = loaded.cases.filter(
  (fixture) => fixture.data.method === "textDocument/documentSymbol",
);

type Symbol = {
  name: string;
  kind: number;
  children?: Symbol[];
  selectionRange: { start: { line: number; character: number } };
};

function symbolReply(messages: JsonRpcMessage[]): Symbol[] {
  const response = messages.find((message) => message.id === 2 && !message.method);
  assert(response && Array.isArray(response.result));
  return response.result as Symbol[];
}

void test("outline cases preserve complete original projects and separately authored arrays", () => {
  assert.equal(originals.length, 2);
  for (const fixture of originals) {
    assert.deepEqual(
      fixture.files.map((file) => file.runtimePath),
      ["src/MySwitch.vue", "src/Parent.vue", "tsconfig.json", "vize.config.json"],
    );
    assert.equal(fixture.requests.length, 1);
    assert.equal(fixture.data.hierarchicalDocumentSymbols, true);
    const result = fixture.requests[0].result as Symbol[];
    assert.deepEqual(
      result.map((symbol) => symbol.name),
      ["template", "script setup"],
    );
    assert(result.every((block) => block.kind === 2 && (block.children?.length ?? 0) > 0));
    const observation = referenceObservation(fixture);
    assert.deepEqual(inspectLspObservation(fixture, observation), [
      { requestId: 2, state: "equal" },
    ]);
  }
});

void test("hierarchy capability is explicit for new sessions and every older case is unchanged", () => {
  for (const fixture of loaded.cases) {
    const capabilities = initializeCapabilities(fixture);
    if (fixture.data.method === "textDocument/documentSymbol") {
      assert.deepEqual(capabilities, {
        textDocument: {
          ...INITIALIZE_CAPABILITIES.textDocument,
          documentSymbol: { hierarchicalDocumentSymbolSupport: true },
        },
      });
    } else {
      assert.deepEqual(capabilities, INITIALIZE_CAPABILITIES);
    }
  }
  const fixture = originals[0];
  const observation = referenceObservation(fixture);
  rewriteFrames(observation, "client", (messages) => {
    const params = messages[0].params;
    assert(params);
    params.capabilities = INITIALIZE_CAPABILITIES;
  });
  assert.throws(() => inspectLspObservation(fixture, observation));
});

void test("whole symbol comparator refuses lost hierarchy, reordered children and changed full fields", () => {
  const fixture = originals.find((fixture) => fixture.entry === "src/Parent.vue") as LspFixture;
  assert(fixture);
  for (const mutate of [
    (symbols: Symbol[]) => {
      delete symbols[0].children;
    },
    (symbols: Symbol[]) => {
      symbols[0].children?.reverse();
    },
    (symbols: Symbol[]) => {
      if (symbols[0].children) symbols[0].children[0].kind = 19;
    },
    (symbols: Symbol[]) => {
      if (symbols[1].children) symbols[1].children[0].name = "other";
    },
    (symbols: Symbol[]) => {
      const child = symbols[1].children?.[0];
      assert(child);
      child.selectionRange.start.character += 1;
    },
    (symbols: Symbol[]) => {
      Object.assign(symbols[0], { extraField: null });
    },
  ]) {
    const observation = referenceObservation(fixture);
    rewriteFrames(observation, "server", (messages) => mutate(symbolReply(messages)));
    assert.deepEqual(inspectLspObservation(fixture, observation), [
      { requestId: 2, state: "different" },
    ]);
  }
});
