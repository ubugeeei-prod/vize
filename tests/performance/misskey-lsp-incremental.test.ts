import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { withPinnedFixtureWorkspace } from "../_helpers/realworld-patch.ts";
import { completionLabels, hoverToText } from "../tooling/support/lsp/assertions.ts";
import { LspSession } from "../tooling/support/lsp/session.ts";
import { countFiles, IncrementalMetrics } from "./support/incremental-metrics.ts";
import {
  assertSingleInjectedMismatch,
  changeVue,
  diagnosticsTimeoutMs,
  normalizeDiagnostics,
  positionInsideTemplateSymbol,
  replaceExactly,
  waitForDiagnostics,
} from "./support/lsp-oracle.ts";

const componentPath = "packages/frontend/src/components/MkDivider.vue";
const dependencyPath = "packages/frontend/src/components/MkCodeInline.vue";
const symbol = "benchmarkMirror";

test(
  "Misskey LSP clean, leaf, and shared edits stay exact in one production session",
  { timeout: 300_000 },
  async () => {
    await withPinnedFixtureWorkspace(
      { fixtureId: "misskey", includePaths: ["packages/frontend"] },
      async (fixture) => {
        const workspaceDir = path.join(fixture.workspaceDir, "packages/frontend");
        const sourceDir = path.join(workspaceDir, "src");
        const componentFile = fixture.resolve(componentPath);
        const componentUri = pathToFileURL(componentFile).href;
        const dependencyFile = fixture.resolve(dependencyPath);
        const dependencyUri = pathToFileURL(dependencyFile).href;
        const cleanDependency = fixture.read(dependencyPath);
        const brokenDependency = replaceExactly(
          cleanDependency,
          "\tcode: string;",
          "\tcode: number;",
        );
        const cleanSource = prepareComponent(fixture.read(componentPath));
        const leafBrokenSource = replaceExactly(
          cleanSource,
          `const ${symbol}: number = 1;`,
          `const ${symbol}: number = 'leaf-broken';`,
        );
        const vueFiles = countFiles(sourceDir, new Set([".vue"]));
        const sourceFiles = countFiles(sourceDir, new Set([".vue", ".ts", ".tsx"]));
        assert.ok(vueFiles >= 500, `expected a monorepo-scale fixture, got ${vueFiles} Vue files`);

        const session = new LspSession();
        const metrics = new IncrementalMetrics(session.processId, {
          id: "misskey-lsp-incremental",
          title: "Misskey LSP Incremental Oracle",
        });
        let baseline: string[] = [];
        let failure: unknown;

        try {
          await metrics.measure("initialize", () =>
            session.initialize(workspaceDir, {
              completion: true,
              editor: true,
              hover: true,
              lint: false,
              typecheck: true,
            }),
          );
          session.notify("textDocument/didOpen", {
            textDocument: {
              uri: dependencyUri,
              languageId: "vue",
              version: 1,
              text: cleanDependency,
            },
          });

          const cleanPublish = await metrics.measure("coldOpen", async () => {
            session.notify("textDocument/didOpen", {
              textDocument: {
                uri: componentUri,
                languageId: "vue",
                version: 1,
                text: cleanSource,
              },
            });
            return waitForDiagnostics(session, componentUri, 1);
          });
          assert.equal(
            cleanPublish.diagnostics.length,
            0,
            `expected no clean diagnostics: ${JSON.stringify(cleanPublish.diagnostics)}`,
          );
          baseline = normalizeDiagnostics(cleanPublish.diagnostics);

          const completion = await metrics.measure("completion", () =>
            session.request(
              "textDocument/completion",
              {
                textDocument: { uri: componentUri },
                position: positionInsideTemplateSymbol(cleanSource, symbol, "benchmarkM"),
              },
              diagnosticsTimeoutMs,
            ),
          );
          assert.ok(
            completionLabels(completion as never).includes(symbol),
            JSON.stringify(completion),
          );

          const hover = (await metrics.measure("hover", () =>
            session.request(
              "textDocument/hover",
              {
                textDocument: { uri: componentUri },
                position: positionInsideTemplateSymbol(cleanSource, symbol, symbol),
              },
              diagnosticsTimeoutMs,
            ),
          )) as { contents?: unknown } | null;
          const hoverText = hoverToText(hover);
          assert.match(hoverText, new RegExp(symbol));
          assert.match(hoverText, /number/i);

          const warmPublish = await changeVue(session, metrics, {
            name: "warmNoop",
            uri: componentUri,
            version: 2,
            source: cleanSource,
          });
          assert.deepEqual(normalizeDiagnostics(warmPublish.diagnostics), baseline);

          const leafBroken = await changeVue(session, metrics, {
            name: "leafBroken",
            uri: componentUri,
            version: 3,
            source: leafBrokenSource,
            expectError: true,
          });
          assertSingleInjectedMismatch(leafBroken.diagnostics, baseline, leafBrokenSource, symbol);

          const leafRepaired = await changeVue(session, metrics, {
            name: "leafRepaired",
            uri: componentUri,
            version: 4,
            source: cleanSource,
          });
          assert.deepEqual(normalizeDiagnostics(leafRepaired.diagnostics), baseline);

          const sharedBroken = await metrics.measure("sharedBroken", async () => {
            session.notify("textDocument/didChange", {
              textDocument: { uri: dependencyUri, version: 2 },
              contentChanges: [{ text: brokenDependency }],
            });
            return waitForDiagnostics(session, componentUri, 4, true);
          });
          assertSingleInjectedMismatch(sharedBroken.diagnostics, baseline, cleanSource, symbol, {
            attributeName: "code",
          });

          const sharedRepaired = await metrics.measure("sharedRepaired", async () => {
            session.notify("textDocument/didChange", {
              textDocument: { uri: dependencyUri, version: 3 },
              contentChanges: [{ text: cleanDependency }],
            });
            return waitForDiagnostics(session, componentUri, 4, false);
          });
          assert.deepEqual(normalizeDiagnostics(sharedRepaired.diagnostics), baseline);

          session.notify("textDocument/didClose", { textDocument: { uri: componentUri } });
          session.notify("textDocument/didClose", { textDocument: { uri: dependencyUri } });
        } catch (error) {
          failure = error;
        }
        try {
          await session.shutdown();
        } catch (error) {
          failure ??= error;
        }
        metrics.write(
          {
            fixture: "misskey/packages/frontend",
            revision: fixture.entry.revision,
            vueFiles,
            sourceFiles,
            baselineDiagnostics: baseline.length,
          },
          failure,
        );
        if (failure != null) throw failure;
      },
    );
  },
);

function prepareComponent(source: string): string {
  let patched = replaceExactly(
    source,
    '<script setup lang="ts">\n',
    `<script setup lang="ts">\nimport MkCodeInline from './MkCodeInline.vue';\n\nconst ${symbol}: number = 1;\n`,
  );
  return replaceExactly(
    patched,
    "</div>\n</template>",
    `\t<MkCodeInline :code="String(${symbol})" /><span>{{ ${symbol} }}</span>\n</div>\n</template>`,
  );
}
