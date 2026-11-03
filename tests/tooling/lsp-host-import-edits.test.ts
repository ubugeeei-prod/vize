import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

const host = (script: string) => `<script setup lang="ts">
${script}
</script>

<template>
  <div>{{ shown }}</div>
</template>
`;

// An edit that keeps every import reuses the host's prepared graph; one that
// adds or removes an import rebuilds it. Both must publish the buffer's errors.
const revisions: { script: string; expected: { code: number; message: string }[] }[] = [
  { script: "import { label } from './util'\nconst shown: string = label", expected: [] },
  {
    script: "import { label } from './util'\nconst shown: number = label",
    expected: [{ code: 2322, message: "Type 'string' is not assignable to type 'number'." }],
  },
  {
    script:
      "import { label } from './util'\nimport { extra } from './extra'\nconst shown: string = label + extra.count.toFixed('x')",
    expected: [
      {
        code: 2345,
        message: "Argument of type 'string' is not assignable to parameter of type 'number'.",
      },
    ],
  },
  {
    script: "import { label } from './util'\nconst shown: string = label + extra.count",
    expected: [{ code: 2304, message: "Cannot find name 'extra'." }],
  },
  {
    script:
      "import { label } from './util'\nimport { nope } from './nope'\nconst shown: string = label + nope",
    expected: [
      {
        code: 2307,
        message: "Cannot find module './nope' or its corresponding type declarations.",
      },
    ],
  },
  { script: "import { label } from './util'\nconst shown: string = label", expected: [] },
];

test("vize lsp publishes exact type diagnostics as a host keeps, adds and removes imports", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "TypeScript 7/Corsa runtime for the host import edit gate",
    "TypeScript 7/Corsa runtime not found; skipping LSP host import edit test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-host-import-edits");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    const sourceDir = path.join(workspaceDir, "src");
    fs.mkdirSync(sourceDir, { recursive: true });
    const vuePackage = path.join(root, "node_modules/vue");
    if (fs.existsSync(vuePackage)) {
      fs.mkdirSync(path.join(workspaceDir, "node_modules"), { recursive: true });
      fs.symlinkSync(vuePackage, path.join(workspaceDir, "node_modules/vue"), "dir");
      const vueScope = path.join(root, "node_modules/@vue");
      if (fs.existsSync(vueScope)) {
        fs.symlinkSync(vueScope, path.join(workspaceDir, "node_modules/@vue"), "dir");
      }
    }
    writeVueConfigFixture(
      workspaceDir,
      JSON.stringify({ lsp: { lint: false, typecheck: true }, typeChecker: { corsaPath } }),
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          strict: true,
          target: "ES2022",
        },
        include: ["src/**/*"],
      }),
    );
    fs.writeFileSync(path.join(sourceDir, "util.ts"), "export const label: string = 'ok'\n");
    fs.writeFileSync(path.join(sourceDir, "extra.ts"), "export const extra = { count: 1 }\n");
    const hostPath = path.join(sourceDir, "Host.vue");
    const hostUri = pathToFileURL(hostPath).href;
    fs.writeFileSync(hostPath, host(revisions[0].script));

    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });
    for (const [index, revision] of revisions.entries()) {
      const version = index + 1;
      const text = host(revision.script);
      if (index === 0) {
        session.notify("textDocument/didOpen", {
          textDocument: { uri: hostUri, languageId: "vue", version, text },
        });
      } else {
        session.notify("textDocument/didChange", {
          textDocument: { uri: hostUri, version },
          contentChanges: [{ text }],
        });
      }
      const published = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) =>
          isDiagnosticsForUri(params, hostUri) &&
          params.version === version &&
          params.diagnostics.length === revision.expected.length,
      )) as PublishDiagnosticsParams;
      assert.deepEqual(
        published.diagnostics.map(({ code, message }) => ({ code, message })),
        revision.expected,
        `revision ${version}`,
      );
    }
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
