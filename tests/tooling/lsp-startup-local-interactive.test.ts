import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import { workspace } from "./support/upstream/vue-language-tools.ts";

test(
  "cold CSS hover and exact local completion stay prompt while native preparation owns its scope",
  { timeout: 30_000 },
  async () => {
    const directory = workspace("startup-local-interactive-");
    const session = new LspSession();
    const uri = pathToFileURL(path.join(directory, "App.vue")).href;
    const source = `<script setup lang="ts">
const palette = { tone: 'warm', count: 1 };
const chosen = palette.tone;
</script>
<template><div>{{ chosen }}</div></template>
<style>.panel { color: red; }</style>
`;
    let timeout: NodeJS.Timeout | undefined;
    try {
      fs.writeFileSync(path.join(directory, "App.vue"), source);
      fs.writeFileSync(path.join(directory, "entry.ts"), "export {};\n");
      fs.writeFileSync(
        path.join(directory, "tsconfig.json"),
        JSON.stringify({
          compilerOptions: {
            strict: true,
            target: "ESNext",
            module: "ESNext",
            moduleResolution: "Bundler",
            skipLibCheck: true,
          },
          include: ["*.ts", "*.vue"],
        }),
      );
      await session.initialize(directory, {
        editor: true,
        lint: false,
        ecosystem: false,
        typecheck: true,
      });
      const preparing = new Promise<void>((resolve, reject) => {
        timeout = setTimeout(
          () => reject(new Error(`native preparation never began\n${session.stderrText}`)),
          10_000,
        );
        session.stderrObservers.push((text) => {
          if (text.includes(`preparing initial native project for ${uri} version 1`)) {
            clearTimeout(timeout);
            resolve();
          }
        });
      });
      const openedAt = performance.now();
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: source },
      });
      await preparing;
      const request = async (method: string, offset: number) => {
        const started = performance.now();
        const response = await session.request(
          method,
          { textDocument: { uri }, position: offsetToPosition(source, offset) },
          1000,
        );
        const elapsed = performance.now() - started;
        process.stdout.write(
          `${JSON.stringify({ method, elapsedMs: elapsed, phase: "initial native preparation" })}\n`,
        );
        assert.ok(
          elapsed < 100,
          `${method} source-local response took ${elapsed.toFixed(1)}ms during native preparation`,
        );
        return response;
      };
      const [hover, completion] = await Promise.all([
        request("textDocument/hover", source.indexOf("color") + 2),
        request("textDocument/completion", source.indexOf("palette.tone") + "palette.".length),
      ]);
      assert.match(hoverToText(hover as { contents?: unknown }), /color/i);
      assert.deepEqual(
        completion,
        ["tone", "count"].map((label) => ({
          label,
          kind: 10,
          detail: "local object member",
          documentation: {
            kind: "markdown",
            value: "Statically declared member of the local object `palette`.",
          },
          sortText: `0${label}`,
        })),
      );
      assert.ok(
        !session.stderrText.includes(`prepared initial native project for ${uri} version 1`),
        "both responses must finish while the actual native scope is still preparing",
      );
      const terminal = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
        10_000,
      )) as PublishDiagnosticsParams;
      assert.deepEqual(terminal.diagnostics, []);
      process.stdout.write(
        `${JSON.stringify({ terminalAfterOpenMs: performance.now() - openedAt, version: terminal.version, diagnostics: terminal.diagnostics })}\n`,
      );
      assert.ok(
        performance.now() - openedAt >= 1000,
        "terminal type diagnostics must preserve the entire initial grace",
      );
      assert.ok(
        session.stderrText.includes(`prepared initial native project for ${uri} version 1`),
      );
    } finally {
      clearTimeout(timeout);
      await session.shutdown();
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);

test(
  "JSX opt-out keeps SFC-looking strings inert and opt-in keeps native type hover",
  { timeout: 30_000 },
  async () => {
    for (const jsxTypecheck of [false, true]) {
      const directory = workspace("startup-jsx-boundary-");
      const session = new LspSession();
      const source = `const value: number = 1;
const raw = \`<script setup lang="ts">
const palette = { tone: 'warm' };
const chosen = palette.tone;
</script><style>.panel { color: red; }</style>\`;
export const view = <div>{value}</div>;
`;
      const uri = pathToFileURL(path.join(directory, "Probe.tsx")).href;
      try {
        fs.writeFileSync(path.join(directory, "Probe.tsx"), source);
        fs.writeFileSync(
          path.join(directory, "vite.config.mjs"),
          `export default { vue: { typeChecker: { jsxTypecheck: ${jsxTypecheck} } } };\n`,
        );
        fs.writeFileSync(
          path.join(directory, "tsconfig.json"),
          JSON.stringify({
            compilerOptions: {
              jsx: "preserve",
              target: "ESNext",
              module: "ESNext",
              moduleResolution: "Bundler",
              skipLibCheck: true,
            },
            include: ["*.tsx"],
          }),
        );
        await session.initialize(directory, {
          editor: true,
          lint: false,
          ecosystem: false,
          typecheck: true,
        });
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "typescriptreact", version: 1, text: source },
        });
        const request = (method: string, offset: number) =>
          session.request(method, {
            textDocument: { uri },
            position: offsetToPosition(source, offset),
          });
        if (!jsxTypecheck) {
          assert.equal(await request("textDocument/hover", source.indexOf("color") + 2), null);
          assert.equal(
            await request(
              "textDocument/completion",
              source.indexOf("palette.tone") + "palette.".length,
            ),
            null,
          );
        } else {
          const hover = await request("textDocument/hover", source.lastIndexOf("value") + 2);
          assert.match(hoverToText(hover as { contents?: unknown }), /value/);
          assert.match(hoverToText(hover as { contents?: unknown }), /number/);
        }
      } finally {
        await session.shutdown();
        fs.rmSync(directory, { recursive: true, force: true });
      }
    }
  },
);
