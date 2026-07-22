import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { withPinnedFixtureWorkspace } from "../_helpers/realworld-patch.ts";
import {
  completionLabels,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "../tooling/support/lsp/assertions.ts";
import type { LspDiagnostic, PublishDiagnosticsParams } from "../tooling/support/lsp/protocol.ts";
import { LspSession } from "../tooling/support/lsp/session.ts";
import { countFiles, IncrementalMetrics } from "./support/incremental-metrics.ts";

const componentPath = "packages/frontend/src/components/MkDivider.vue";
const dependencyPath = "packages/frontend/src/components/MkCodeInline.vue";
const symbol = "benchmarkMirror";
const waitTimeoutMs = 120_000;

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
        const metrics = new IncrementalMetrics(session.processId);
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
                position: positionInsideTemplateSymbol(cleanSource, "benchmarkM"),
              },
              waitTimeoutMs,
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
                position: positionInsideTemplateSymbol(cleanSource, symbol),
              },
              waitTimeoutMs,
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
          assertSingleInjectedMismatch(leafBroken.diagnostics, baseline, leafBrokenSource);

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
          assertSingleInjectedMismatch(
            sharedBroken.diagnostics,
            baseline,
            cleanSource,
            "attribute-name",
          );

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

function replaceExactly(source: string, expected: string, replacement: string): string {
  const first = source.indexOf(expected);
  assert.notEqual(first, -1, `missing patch anchor: ${expected}`);
  assert.equal(
    source.indexOf(expected, first + expected.length),
    -1,
    "patch anchor must be unique",
  );
  return `${source.slice(0, first)}${replacement}${source.slice(first + expected.length)}`;
}

async function changeVue(
  session: LspSession,
  metrics: IncrementalMetrics,
  change: { name: string; uri: string; version: number; source: string; expectError?: boolean },
): Promise<PublishDiagnosticsParams> {
  return metrics.measure(change.name, async () => {
    session.notify("textDocument/didChange", {
      textDocument: { uri: change.uri, version: change.version },
      contentChanges: [{ text: change.source }],
    });
    return waitForDiagnostics(session, change.uri, change.version, change.expectError);
  });
}

async function waitForDiagnostics(
  session: LspSession,
  uri: string,
  version: number,
  expectError?: boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => {
      if (!isDiagnosticsForUri(params, uri) || params.version !== version) return false;
      return expectError == null || hasInjectedMismatch(params.diagnostics) === expectError;
    },
    waitTimeoutMs,
  )) as PublishDiagnosticsParams;
}

function hasInjectedMismatch(diagnostics: LspDiagnostic[]): boolean {
  return diagnostics.some(
    (diagnostic) =>
      String(diagnostic.code).replace(/^TS/, "") === "2322" &&
      /string.*not assignable.*number/i.test(diagnostic.message ?? ""),
  );
}

function assertSingleInjectedMismatch(
  diagnostics: LspDiagnostic[],
  baseline: string[],
  source: string,
  expectedRange: "declaration" | "attribute-name" = "declaration",
): void {
  const injected = diagnostics.filter(
    (diagnostic) =>
      String(diagnostic.code).replace(/^TS/, "") === "2322" &&
      /string.*not assignable.*number/i.test(diagnostic.message ?? ""),
  );
  assert.equal(injected.length, 1, JSON.stringify(diagnostics));
  const [diagnostic] = injected;
  assert.equal(diagnostic.source, "vize/types");
  assert.equal(diagnostic.severity, 1);
  if (expectedRange === "declaration") {
    const declarationOffset = source.indexOf(`const ${symbol}`);
    assert.notEqual(declarationOffset, -1);
    const start = offsetToPosition(source, declarationOffset + "const ".length);
    const end = { line: start.line, character: start.character + symbol.length };
    assert.deepEqual(diagnostic.range?.start, start);
    assert.deepEqual(diagnostic.range?.end, end);
  } else {
    // The child prop-type mismatch anchors at the attribute name, exactly
    // where vue-tsc reports it.
    const attributeOffset = source.indexOf(`:code="String(${symbol})"`);
    assert.notEqual(attributeOffset, -1);
    const start = offsetToPosition(source, attributeOffset + ":".length);
    const end = { line: start.line, character: start.character + "code".length };
    assert.deepEqual(diagnostic.range?.start, start);
    assert.deepEqual(diagnostic.range?.end, end);
  }
  assert.deepEqual(
    normalizeDiagnostics(diagnostics.filter((item) => item !== diagnostic)),
    baseline,
  );
}

function normalizeDiagnostics(diagnostics: LspDiagnostic[]): string[] {
  return diagnostics
    .map((diagnostic) =>
      JSON.stringify({
        code: diagnostic.code,
        message: diagnostic.message,
        range: diagnostic.range,
        severity: diagnostic.severity,
        source: diagnostic.source,
      }),
    )
    .sort();
}

function positionInsideTemplateSymbol(
  source: string,
  prefix: string,
): { line: number; character: number } {
  const templateOffset = source.indexOf(`{{ ${symbol} }}`);
  assert.notEqual(templateOffset, -1);
  return offsetToPosition(source, templateOffset + "{{ ".length + prefix.length);
}
