import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { resolveTsgoBinary } from "../../_helpers/realworld-typecheck.ts";
import {
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "../../tooling/support/lsp/assertions.ts";
import { root, testOutputRoot } from "../../tooling/support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

// LSP Ref-unwrap probe (#3283): the upstream vue-benchmarks IDE lane showed a
// Corsa/tsgo startup failure can leave the LSP answering hovers through the
// script-binding heuristic, so a hover latency is not evidence of type
// checking. This oracle pins the discriminators a benchmark needs:
//
// 1. backend liveness is proven by an exact `vize/types` diagnostic round
//    trip (broken -> clean), which the heuristic layer cannot produce;
// 2. a heuristic hover is identifiable: `_Script binding_` /
//    `_Template binding from script_` provenance markers, the script-side
//    `Ref<string>` shown at template positions, and `MaybeRef<unknown>` for a
//    plain `string` const;
// 3. a backend-missing session is identifiable: the exact
//    `typecheck-unavailable` hint diagnostic, and hovers that DECLINE the type
//    rather than guessing one.
//
// The probe REJECTS (refuses to rank) any hover carrying a heuristic or
// backend-missing signature. Since #3321 a live session answers template
// hovers from the backend, so the ranked classification is the required
// behavior and the heuristic signatures must no longer appear at all.
const cleanSource = `<script setup lang="ts">
import { ref } from 'vue'
const message = ref('hello')
const upper = message.value.toUpperCase()
</script>

<template>
  <p>{{ message.toUpperCase() }}{{ upper }}</p>
</template>
`;
const brokenSource = cleanSource.replace(
  "const upper = message.value.toUpperCase()",
  "const upper: number = message.value",
);

const heuristicMarkers = ["_Script binding_", "_Template binding from script_"];

const typecheckUnavailableHint = {
  code: "typecheck-unavailable",
  message:
    "Type checking is unavailable in this workspace. Make sure `tsconfig.json` exists. Install `typescript@^7` or configure `typeChecker.corsaPath`; see https://vizejs.dev/guide/static-analysis.",
  range: {
    start: { line: 0, character: 0 },
    end: { line: 0, character: 0 },
  },
  severity: 4,
  source: "vize/types",
};

type HoverClass = "backend-template-type" | "heuristic" | "empty";

/**
 * Classify a template-position hover. Only a backend-typed answer may be
 * ranked as typecheck evidence; heuristic and empty answers are rejected.
 */
function classifyTemplateHover(hoverText: string, binding: string): HoverClass {
  if (hoverText.trim() === "") return "empty";
  if (heuristicMarkers.some((marker) => hoverText.includes(marker))) return "heuristic";
  if (hoverText.includes("MaybeRef<unknown>")) return "heuristic";
  if (hoverText.includes(`${binding}: string`) && !hoverText.includes("Ref<")) {
    return "backend-template-type";
  }
  return "heuristic";
}

test("ref-unwrap probe proves backend liveness and rejects heuristic hovers", async () => {
  assert.equal(
    process.env.VIZE_TEST_REQUIRE_TSGO,
    "1",
    "the ref-unwrap probe must run with VIZE_TEST_REQUIRE_TSGO=1",
  );
  const corsaPath = resolveTsgoBinary();
  const workspaceDir = createWorkspace("live", corsaPath);
  const filePath = path.join(workspaceDir, "App.vue");
  const uri = pathToFileURL(filePath).href;
  fs.writeFileSync(filePath, brokenSource, "utf8");

  const session = new LspSession();
  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: brokenSource },
    });

    // Liveness gate: the heuristic layer cannot emit this exact `vize/types`
    // TS2322, so a session that never publishes it must not be ranked.
    const broken = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        (params as PublishDiagnosticsParams).diagnostics.length > 0,
      120_000,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(broken, {
      diagnostics: [
        {
          code: 2322,
          message: "Type 'string' is not assignable to type 'number'.",
          range: {
            start: { line: 3, character: 6 },
            end: { line: 3, character: 11 },
          },
          severity: 1,
          source: "vize/types",
        },
      ],
      uri,
      version: 1,
    });

    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 2 },
      contentChanges: [{ text: cleanSource }],
    });
    const repaired = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) && (params as PublishDiagnosticsParams).version === 2,
      120_000,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(repaired, { diagnostics: [], uri, version: 2 });

    // Script-side hover: `Ref<string>` is the correct script type, and it now
    // arrives from the backend rather than the script-binding analysis.
    const scriptHover = await hoverText(
      session,
      uri,
      cleanSource.indexOf("const message") + "const mes".length,
    );
    assert.match(scriptHover, /const message: Ref<string, string>/);
    assert.match(scriptHover, /^```typescript\n/);
    assert.doesNotMatch(scriptHover, /_Script binding_/);

    // Template-side hovers: the backend presents the unwrapped `string` at the
    // interpolation position and the real `string` for the plain const, so both
    // rank as backend evidence and neither may carry a heuristic signature.
    const templateMessageHover = await hoverText(
      session,
      uri,
      cleanSource.indexOf("{{ message") + "{{ mes".length,
    );
    assert.match(templateMessageHover, /const message: string/);
    assert.doesNotMatch(templateMessageHover, /_Template binding from script_/);
    assert.doesNotMatch(templateMessageHover, /Ref</);
    assert.equal(classifyTemplateHover(templateMessageHover, "message"), "backend-template-type");

    const templateUpperHover = await hoverText(
      session,
      uri,
      cleanSource.lastIndexOf("upper }}") + 2,
    );
    assert.match(templateUpperHover, /const upper: string/);
    assert.doesNotMatch(templateUpperHover, /MaybeRef<unknown>/);
    assert.equal(classifyTemplateHover(templateUpperHover, "upper"), "backend-template-type");

    // The classifier itself must accept only the unwrapped backend shape.
    assert.equal(
      classifyTemplateHover("**message**\n\n```typescript\nmessage: string\n```", "message"),
      "backend-template-type",
    );
    assert.equal(classifyTemplateHover("", "message"), "empty");
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});

test("ref-unwrap probe rejects backend-missing sessions by their exact hint", async () => {
  const workspaceDir = createWorkspace("missing", path.join(testOutputRoot, "no-such-tsgo"));
  const filePath = path.join(workspaceDir, "App.vue");
  const uri = pathToFileURL(filePath).href;
  fs.writeFileSync(filePath, cleanSource, "utf8");

  const session = new LspSession();
  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: cleanSource },
    });

    const publish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        (params as PublishDiagnosticsParams).diagnostics.some(
          (diagnostic) => diagnostic.code === "typecheck-unavailable",
        ),
      120_000,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(publish, { diagnostics: [typecheckUnavailableHint], uri, version: 1 });

    // The session still answers hovers, but it DECLINES the type: a typecheck
    // session whose backend never came up hands back documentation only. The
    // heuristic type signature that used to appear here is what made a hover
    // latency look like a measurement of type checking.
    const templateHover = await hoverText(
      session,
      uri,
      cleanSource.indexOf("{{ message") + "{{ mes".length,
    );
    assert.match(templateHover, /_Template binding_/);
    assert.doesNotMatch(templateHover, /_Template binding from script_/);
    assert.doesNotMatch(templateHover, /```typescript/);
    assert.equal(classifyTemplateHover(templateHover, "message"), "heuristic");
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});

// Closed gap (#3283 / #3321): the template hover now reports the template-scope
// type the backend computes — `string`, the unwrapped ref — together with the
// authored-coordinate range of the identifier under the cursor. The range is
// the second half of the claim: a hover carrying virtual-TS coordinates would
// highlight the wrong span even with the right text.
test("template hover presents the backend-unwrapped string type", async () => {
  const corsaPath = resolveTsgoBinary();
  const workspaceDir = createWorkspace("unwrapped", corsaPath);
  const filePath = path.join(workspaceDir, "App.vue");
  const uri = pathToFileURL(filePath).href;
  fs.writeFileSync(filePath, cleanSource, "utf8");

  const session = new LspSession();
  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: cleanSource },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      120_000,
    );

    const message = await hover(session, uri, cleanSource.indexOf("{{ message") + "{{ mes".length);
    assert.match(hoverToText(message), /const message: string/);
    assert.deepEqual(message?.range, {
      start: { line: 7, character: 8 },
      end: { line: 7, character: 15 },
    });

    const upper = await hover(session, uri, cleanSource.lastIndexOf("upper }}") + 2);
    assert.match(hoverToText(upper), /const upper: string/);
    assert.deepEqual(upper?.range, {
      start: { line: 7, character: 35 },
      end: { line: 7, character: 40 },
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});

type HoverResponse = { contents?: unknown; range?: unknown } | null;

async function hover(session: LspSession, uri: string, offset: number): Promise<HoverResponse> {
  return (await session.request("textDocument/hover", {
    textDocument: { uri },
    position: offsetToPosition(cleanSource, offset),
  })) as HoverResponse;
}

async function hoverText(session: LspSession, uri: string, offset: number): Promise<string> {
  return hoverToText(await hover(session, uri, offset));
}

function createWorkspace(label: string, corsaPath: string): string {
  const outputRoot = path.join(testOutputRoot, "vue-benchmarks-lsp-ref-unwrap");
  fs.mkdirSync(outputRoot, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(outputRoot, `${label}-`));

  const vuePackage = [
    path.join(root, "node_modules/vue"),
    path.join(root, "tests/node_modules/vue"),
  ].find((candidate) => fs.existsSync(candidate));
  assert.ok(vuePackage, "Vue package is required for the ref-unwrap probe");
  symlink(vuePackage, path.join(workspaceDir, "node_modules/vue"));
  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    symlink(vueNamespace, path.join(workspaceDir, "node_modules/@vue"));
  }

  fs.writeFileSync(
    path.join(workspaceDir, "vize.config.json"),
    JSON.stringify({ typeChecker: { corsaPath } }, null, 2),
  );
  fs.writeFileSync(
    path.join(workspaceDir, "package.json"),
    JSON.stringify({ name: `ref-unwrap-${label}`, private: true, type: "module" }),
  );
  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          lib: ["ES2022", "DOM", "DOM.Iterable"],
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          skipLibCheck: true,
          strict: true,
          target: "ES2022",
        },
        include: ["**/*.vue"],
      },
      null,
      2,
    ),
  );
  return workspaceDir;
}

function symlink(source: string, target: string): void {
  fs.rmSync(target, { force: true, recursive: true });
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.symlinkSync(source, target, process.platform === "win32" ? "junction" : "dir");
}
