import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { LspRange, ServerCapabilities } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const source = `<script setup lang="ts">
function leaf(value: string): string {
  return value
}

function caller(): string {
  return leaf("setup")
}
</script>

<template>
  <button @click="caller()">{{ leaf("template") }}</button>
</template>
`;

type CallHierarchyCapabilities = ServerCapabilities & {
  callHierarchyProvider?: unknown;
};

type CallHierarchyItem = {
  name: string;
  uri: string;
  range: LspRange;
  selectionRange: LspRange;
  data?: unknown;
};

type IncomingCall = {
  from: CallHierarchyItem;
  fromRanges: LspRange[];
};

type OutgoingCall = {
  to: CallHierarchyItem;
  fromRanges: LspRange[];
};

test("vize lsp maps authored SFC call hierarchy items and call sites", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-call-hierarchy-capability");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
        },
        include: ["**/*"],
      },
      null,
      2,
    ),
  );
  const filePath = path.join(workspaceDir, "App.vue");
  const uri = pathToFileURL(filePath).href;
  const session = new LspSession();

  try {
    const init = (await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    })) as { capabilities?: CallHierarchyCapabilities };
    assert.equal(init.capabilities?.callHierarchyProvider, true);

    fs.writeFileSync(filePath, source, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      60_000,
    );

    const caller = await prepareSingle(session, uri, markerPosition("caller():"));
    assert.equal(caller.name, "caller");
    assert.equal(caller.uri, uri);
    assert.equal(rangeText(source, caller.selectionRange), "caller");
    assert.ok(!caller.uri.endsWith(".vue.ts"), JSON.stringify(caller));

    const outgoing = (await session.request("callHierarchy/outgoingCalls", {
      item: caller,
    })) as OutgoingCall[];
    const leafCall = outgoing.find((call) => call.to.name === "leaf");
    assert.ok(leafCall, JSON.stringify(outgoing));
    assert.equal(leafCall.to.uri, uri);
    assert.equal(rangeText(source, leafCall.to.selectionRange), "leaf");
    assert.deepEqual(
      leafCall.fromRanges.map((range) => rangeText(source, range)),
      ["leaf"],
    );
    assert.ok(!leafCall.to.uri.endsWith(".vue.ts"), JSON.stringify(outgoing));

    const leaf = await prepareSingle(session, uri, markerPosition("leaf(value"));
    const incoming = (await session.request("callHierarchy/incomingCalls", {
      item: leaf,
    })) as IncomingCall[];
    const callerEntry = incoming.find((call) => call.from.name === "caller");
    assert.ok(callerEntry, JSON.stringify(incoming));
    assert.equal(callerEntry.from.uri, uri);
    assert.deepEqual(
      callerEntry.fromRanges.map((range) => rangeText(source, range)),
      ["leaf"],
    );
    assert.ok(!callerEntry.from.uri.endsWith(".vue.ts"), JSON.stringify(incoming));
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

async function prepareSingle(
  session: LspSession,
  uri: string,
  position: { line: number; character: number },
): Promise<CallHierarchyItem> {
  const items = (await session.request("textDocument/prepareCallHierarchy", {
    textDocument: { uri },
    position,
  })) as CallHierarchyItem[];
  assert.ok(Array.isArray(items), JSON.stringify(items));
  assert.equal(items.length, 1, JSON.stringify(items));
  return items[0] as CallHierarchyItem;
}

function markerPosition(marker: string): { line: number; character: number } {
  const offset = source.indexOf(marker);
  assert.notEqual(offset, -1, marker);
  return offsetToPosition(source, offset + 1);
}

function rangeText(text: string, range: LspRange): string {
  return text.slice(positionToOffset(text, range.start), positionToOffset(text, range.end));
}

function positionToOffset(text: string, position: { line: number; character: number }): number {
  let offset = 0;
  for (const line of text.split("\n").slice(0, position.line)) {
    offset += line.length + 1;
  }
  return offset + position.character;
}
