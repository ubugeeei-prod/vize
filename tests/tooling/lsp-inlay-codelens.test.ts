import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type InlayHint = {
  position: { line: number; character: number };
  label: string | Array<{ value: string }>;
  kind?: number;
  paddingLeft?: boolean;
  paddingRight?: boolean;
  tooltip?: string | { value: string };
};

type CodeLens = {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  command?: { title: string; command: string };
};

function inlayLabel(hint: InlayHint): string {
  if (typeof hint.label === "string") {
    return hint.label;
  }
  return hint.label.map((part) => part.value).join("");
}

function inlayTooltip(hint: InlayHint): string | undefined {
  if (hint.tooltip == null) {
    return undefined;
  }
  return typeof hint.tooltip === "string" ? hint.tooltip : hint.tooltip.value;
}

const FULL_RANGE = {
  start: { line: 0, character: 0 },
  end: { line: 1000, character: 0 },
};

test("vize lsp inlayHint surfaces reactive-binding type hints for ref()/computed()", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-inlay-reactive");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    const source = `<script setup lang="ts">
import { ref, computed } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>
`;
    const filePath = path.join(workspaceDir, "Reactive.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    const hints = (await session.request("textDocument/inlayHint", {
      textDocument: { uri },
      range: FULL_RANGE,
    })) as InlayHint[] | null;

    assert.ok(Array.isArray(hints), JSON.stringify(hints));

    const refHint = hints.find((hint) => inlayLabel(hint) === ": Ref<number>");
    const computedHint = hints.find((hint) => inlayLabel(hint) === ": ComputedRef<number>");

    assert.ok(refHint, `expected ": Ref<number>" hint, got ${JSON.stringify(hints)}`);
    assert.ok(computedHint, `expected ": ComputedRef<number>" hint, got ${JSON.stringify(hints)}`);

    assert.equal(refHint.kind, 1);
    assert.equal(computedHint.kind, 1);
    assert.equal(refHint.paddingLeft, true);
    assert.equal(computedHint.paddingLeft, true);
    assert.equal(inlayTooltip(refHint), "Vue reactive binding (Ref)");
    assert.equal(inlayTooltip(computedHint), "Vue reactive binding (ComputedRef)");

    // `const count = ref(0)` sits on line index 2 of the SFC.
    const countLine = source.split("\n").findIndex((line) => line.includes("const count"));
    assert.equal(refHint.position.line, countLine);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp inlayHint renders i18n message preview for $t() keys", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-inlay-i18n");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    const source = `<template>
  <p>{{ $t("auth.login") }}</p>
</template>
<i18n lang="json">
{
  "en": { "auth": { "login": "Log in" } }
}
</i18n>
`;
    const filePath = path.join(workspaceDir, "I18n.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    const hints = (await session.request("textDocument/inlayHint", {
      textDocument: { uri },
      range: FULL_RANGE,
    })) as InlayHint[] | null;

    assert.ok(Array.isArray(hints), JSON.stringify(hints));
    const labels = hints.map(inlayLabel);
    assert.ok(labels.includes("= Log in"), labels.join(", "));
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp codeLens reports per-binding reference counts with singular/plural wording", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-codelens");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    const source = `<script setup lang="ts">
const count = ref(0)
const doubled = ref(0)
function inc() {}
const twice = ref(0)
const unused = ref(0)
</script>

<template>
  <button @click="inc">{{ count }} {{ doubled }}</button>
  <p>{{ twice }} {{ twice }}</p>
</template>
`;
    const filePath = path.join(workspaceDir, "Lens.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    const lenses = (await session.request("textDocument/codeLens", {
      textDocument: { uri },
    })) as CodeLens[] | null;

    assert.ok(Array.isArray(lenses), JSON.stringify(lenses));

    // Characterized current behavior: the lens range is anchored one line BELOW
    // the declaration it describes (the server emits `base_line + line - 1`
    // where `base_line` is already the script-tag line). Map each lens to the
    // declaration immediately above its range to read the binding name.
    const sourceLines = source.split("\n");
    const byName = new Map<string, CodeLens>();
    for (const lens of lenses) {
      const declLine = sourceLines[lens.range.start.line - 1] ?? "";
      const match = declLine.match(/^(?:const|let|function)\s+([A-Za-z0-9_$]+)/);
      if (match) {
        byName.set(match[1], lens);
      }
    }

    const expectedDeclLine = (name: string): number =>
      sourceLines.findIndex((line) =>
        new RegExp(`^(?:const|let|function)\\s+${name}\\b`).test(line),
      );

    // Once-referenced bindings -> "1 reference".
    for (const name of ["count", "doubled", "inc"]) {
      const lens = byName.get(name);
      assert.ok(lens, `expected a lens for ${name}, got ${JSON.stringify(lenses)}`);
      assert.equal(lens.command?.command, "vize.findReferences");
      assert.equal(lens.command?.title, "1 reference");
      assert.equal(lens.range.start.character, 0);
      assert.equal(lens.range.start.line, lens.range.end.line);
      // The lens sits exactly one line below the declaration it annotates.
      assert.equal(lens.range.start.line, expectedDeclLine(name) + 1);
    }

    // Twice-referenced binding -> "2 references".
    const twice = byName.get("twice");
    assert.ok(twice, JSON.stringify(lenses));
    assert.equal(twice.command?.title, "2 references");

    // Zero-reference binding -> no lens at all.
    assert.equal(byName.has("unused"), false, JSON.stringify(lenses));
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
