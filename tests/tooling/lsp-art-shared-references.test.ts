import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

test("art references join shared script identity across current variants without merging shadowed names", async (t) => {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "art-shared-references-"));
  const session = new LspSession();
  const timings: number[] = [];
  try {
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          target: "ESNext",
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
        },
        include: ["./**/*"],
      }),
    );
    fs.writeFileSync(path.join(workspace, "Child.vue"), "<template><slot /></template>\n");
    await session.initialize(workspace, { editor: true, lint: false, typecheck: true });
    for (const inline of [false, true]) {
      for (const newline of ["\n", "\r\n"]) {
        const ordinary = inline ? "<template><main>{{ shared }}</main></template>\n" : "";
        const source = `<script setup lang="ts">
const shared = 'outer'
const rows = [shared]
function echo(shared: string) { return shared }
</script>
${ordinary}<art title="Scope" component="./Child.vue">
<variant name="First" default>
<p>🧭{{ shared }}</p><i v-for="shared in rows">{{ shared }}</i>
</variant>
<variant name="Second">
<section>🧭{{ shared }}</section><Child v-slot="{ shared }">{{ shared }}</Child>
</variant>
<variant name="Third"><footer>{{ shared }}</footer></variant>
</art>
`.replaceAll("\n", newline);
        const suffix = `${inline ? "Inline" : "Separate"}-${newline.length}.${inline ? "vue" : "art.vue"}`;
        // Equal text in a different file is a different authored binding.
        const foreignFile = path.join(workspace, `Foreign-${suffix}`);
        const foreignUri = pathToFileURL(foreignFile).href;
        fs.writeFileSync(foreignFile, source);
        session.notify("textDocument/didOpen", {
          textDocument: {
            uri: foreignUri,
            languageId: inline ? "vue" : "art-vue",
            version: 1,
            text: source,
          },
        });
        const file = path.join(workspace, suffix);
        const uri = pathToFileURL(file).href;
        fs.writeFileSync(file, source);
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: inline ? "vue" : "art-vue", version: 1, text: source },
        });
        await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
          isDiagnosticsForUri(params, uri),
        );

        async function verify(content: string, targetUri = uri) {
          const at = (marker: string, prefix: string) => {
            const offset = content.indexOf(marker);
            assert.notEqual(offset, -1, `missing independent source marker ${marker}`);
            return offset + prefix.length;
          };
          const declaration = at("const shared =", "const ");
          const globalUsages = [
            at("[shared]", "["),
            at("<p>🧭{{ shared }}", "<p>🧭{{ "),
            at("<section>🧭{{ shared }}", "<section>🧭{{ "),
          ];
          if (inline) globalUsages.push(at("<main>{{ shared }}", "<main>{{ "));
          if (content.includes("<footer>{{ shared }}"))
            globalUsages.push(at("<footer>{{ shared }}", "<footer>{{ "));
          const location = (offset: number) => ({
            uri: targetUri,
            range: {
              start: offsetToPosition(content, offset),
              end: offsetToPosition(content, offset + "shared".length),
            },
          });
          const localPairs = [
            [at('v-for="shared in rows"', 'v-for="'), at("{{ shared }}</i>", "{{ ")],
            [at('v-slot="{ shared }"', 'v-slot="{ '), at("{{ shared }}</Child>", "{{ ")],
            [at("echo(shared:", "echo("), at("return shared", "return ")],
          ];
          for (const includeDeclaration of [true, false]) {
            const expected = [...(includeDeclaration ? [declaration] : []), ...globalUsages]
              .sort((a, b) => a - b)
              .map(location);
            // Query from the physical declaration, every variant, and the
            // ordinary template rather than relying on an earlier open query.
            for (const offset of [declaration, ...globalUsages]) {
              const started = performance.now();
              const result = await session.request("textDocument/references", {
                textDocument: { uri: targetUri },
                position: offsetToPosition(content, offset + "shared".length),
                context: { includeDeclaration },
              });
              timings.push(performance.now() - started);
              assert.deepEqual(
                result,
                expected,
                `${suffix}: complete shared references, includeDeclaration=${includeDeclaration}`,
              );
            }
            for (const [binding, usage] of localPairs) {
              assert.deepEqual(
                await session.request("textDocument/references", {
                  textDocument: { uri: targetUri },
                  position: offsetToPosition(content, usage),
                  context: { includeDeclaration },
                }),
                [...(includeDeclaration ? [binding] : []), usage].map(location),
                `${suffix}: complete local references must exclude outer, sibling and foreign bindings`,
              );
            }
          }
        }
        await verify(source, foreignUri);
        await verify(source);
        // Shift authored ranges and remove an occurrence while old sibling
        // projections are already open in the native session.
        const changed = source
          .replace("<art title=", `<!-- edited Ω🧭 -->${newline}<art title=`)
          .replace("<footer>{{ shared }}</footer>", "<footer>removed</footer>");
        session.notify("textDocument/didChange", {
          textDocument: { uri, version: 2 },
          contentChanges: [{ text: changed }],
        });
        await verify(changed);
      }
    }
    timings.sort((a, b) => a - b);
    t.diagnostic(
      JSON.stringify({
        sharedReferenceRequests: timings.length,
        p50Ms: timings[Math.floor(timings.length / 2)],
        maxMs: timings.at(-1),
        scope: "source correctness workload timings; no cross-platform speed claim",
      }),
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
