import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { withPinnedFixtureWorkspace } from "../_helpers/realworld-patch.ts";
import { completionLabels, hoverToText } from "../tooling/support/lsp/assertions.ts";
import { LspSession } from "../tooling/support/lsp/session.ts";
import { assertCancellationWindow, recordPublishes } from "./support/churn-oracle.ts";
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

// A shared `packages/` component that every Vben app renders through the basic
// layout, plus one real view in each of two different `apps/` workspace
// packages. Editing the shared component must refresh diagnostics in both apps.
const sharedComponentPath = "packages/effects/layouts/src/basic/copyright/copyright.vue";
const firstLeafPath = "apps/web-antd/src/views/_core/fallback/offline.vue";
const secondLeafPath = "apps/web-ele/src/views/_core/fallback/offline.vue";
const sharedComponentImport =
  "../../../../../../packages/effects/layouts/src/basic/copyright/copyright.vue";
const symbol = "monorepoMirror";
const ignoredDirectories = new Set(["node_modules"]);

test(
  "Vben LSP cross-package edits stay exact across two apps in one production session",
  { timeout: 300_000 },
  async () => {
    await withPinnedFixtureWorkspace(
      {
        fixtureId: "vue-vben-admin",
        includePaths: ["apps", "packages", "playground", "package.json", "pnpm-workspace.yaml"],
      },
      async (fixture) => {
        const workspaceDir = fixture.workspaceDir;
        const sharedUri = pathToFileURL(fixture.resolve(sharedComponentPath)).href;
        const firstLeafUri = pathToFileURL(fixture.resolve(firstLeafPath)).href;
        const secondLeafUri = pathToFileURL(fixture.resolve(secondLeafPath)).href;
        const cleanShared = prepareSharedComponent(fixture.read(sharedComponentPath));
        const brokenShared = replaceExactly(cleanShared, "  date?: string;", "  date?: number;");
        const firstLeafSource = prepareLeaf(fixture.read(firstLeafPath));
        const secondLeafSource = prepareLeaf(fixture.read(secondLeafPath));
        const leafBrokenSource = replaceExactly(
          firstLeafSource,
          `const ${symbol}: number = 1;`,
          `const ${symbol}: number = 'leaf-broken';`,
        );
        const vueExtension = new Set([".vue"]);
        const vueFiles = countFiles(workspaceDir, vueExtension, ignoredDirectories);
        const sourceFiles = countFiles(
          workspaceDir,
          new Set([".vue", ".ts", ".tsx"]),
          ignoredDirectories,
        );
        const appVueFiles = countFiles(
          path.join(workspaceDir, "apps"),
          vueExtension,
          ignoredDirectories,
        );
        const packageVueFiles = countFiles(
          path.join(workspaceDir, "packages"),
          vueExtension,
          ignoredDirectories,
        );
        assert.ok(vueFiles >= 500, `expected a monorepo-scale fixture, got ${vueFiles} Vue files`);
        assert.ok(appVueFiles >= 100, `expected app-side scale, got ${appVueFiles} Vue files`);
        assert.ok(
          packageVueFiles >= 250,
          `expected package-side scale, got ${packageVueFiles} Vue files`,
        );

        const session = new LspSession();
        const metrics = new IncrementalMetrics(session.processId, {
          id: "vben-lsp-incremental",
          title: "Vue Vben Admin LSP Incremental Oracle",
        });
        const publishes = recordPublishes(session);
        let baseline: string[] = [];
        let secondBaseline: string[] = [];
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
              uri: sharedUri,
              languageId: "vue",
              version: 1,
              text: cleanShared,
            },
          });

          const cleanPublish = await metrics.measure("coldOpen", async () => {
            session.notify("textDocument/didOpen", {
              textDocument: {
                uri: firstLeafUri,
                languageId: "vue",
                version: 1,
                text: firstLeafSource,
              },
            });
            return waitForDiagnostics(session, firstLeafUri, 1);
          });
          assert.equal(
            cleanPublish.diagnostics.length,
            0,
            `expected no clean diagnostics: ${JSON.stringify(cleanPublish.diagnostics)}`,
          );
          baseline = normalizeDiagnostics(cleanPublish.diagnostics);

          const secondPublish = await metrics.measure("coldOpenSecondApp", async () => {
            session.notify("textDocument/didOpen", {
              textDocument: {
                uri: secondLeafUri,
                languageId: "vue",
                version: 1,
                text: secondLeafSource,
              },
            });
            return waitForDiagnostics(session, secondLeafUri, 1);
          });
          assert.equal(
            secondPublish.diagnostics.length,
            0,
            `expected no clean diagnostics: ${JSON.stringify(secondPublish.diagnostics)}`,
          );
          secondBaseline = normalizeDiagnostics(secondPublish.diagnostics);

          const completion = await metrics.measure("completion", () =>
            session.request(
              "textDocument/completion",
              {
                textDocument: { uri: firstLeafUri },
                position: positionInsideTemplateSymbol(firstLeafSource, symbol, "monorepoM"),
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
                textDocument: { uri: firstLeafUri },
                position: positionInsideTemplateSymbol(firstLeafSource, symbol, symbol),
              },
              diagnosticsTimeoutMs,
            ),
          )) as { contents?: unknown } | null;
          const hoverText = hoverToText(hover);
          assert.match(hoverText, new RegExp(symbol));
          assert.match(hoverText, /number/i);

          const warmPublish = await changeVue(session, metrics, {
            name: "warmNoop",
            uri: firstLeafUri,
            version: 2,
            source: firstLeafSource,
          });
          assert.deepEqual(normalizeDiagnostics(warmPublish.diagnostics), baseline);

          const leafBroken = await changeVue(session, metrics, {
            name: "leafBroken",
            uri: firstLeafUri,
            version: 3,
            source: leafBrokenSource,
            expectError: true,
          });
          assertSingleInjectedMismatch(leafBroken.diagnostics, baseline, leafBrokenSource, symbol);
          const leafBrokenBaseline = normalizeDiagnostics(leafBroken.diagnostics);

          const leafRepaired = await changeVue(session, metrics, {
            name: "leafRepaired",
            uri: firstLeafUri,
            version: 4,
            source: firstLeafSource,
          });
          assert.deepEqual(normalizeDiagnostics(leafRepaired.diagnostics), baseline);

          const sharedBroken = await metrics.measure("sharedBroken", async () => {
            session.notify("textDocument/didChange", {
              textDocument: { uri: sharedUri, version: 2 },
              contentChanges: [{ text: brokenShared }],
            });
            return waitForDiagnostics(session, firstLeafUri, 4, true);
          });
          assertSingleInjectedMismatch(
            sharedBroken.diagnostics,
            baseline,
            firstLeafSource,
            symbol,
            {
              attributeName: "date",
            },
          );
          const sharedBrokenSecond = await metrics.measure("sharedBrokenSecondApp", () =>
            waitForDiagnostics(session, secondLeafUri, 1, true),
          );
          assertSingleInjectedMismatch(
            sharedBrokenSecond.diagnostics,
            secondBaseline,
            secondLeafSource,
            symbol,
            { attributeName: "date" },
          );

          const sharedRepaired = await metrics.measure("sharedRepaired", async () => {
            session.notify("textDocument/didChange", {
              textDocument: { uri: sharedUri, version: 3 },
              contentChanges: [{ text: cleanShared }],
            });
            return waitForDiagnostics(session, firstLeafUri, 4, false);
          });
          assert.deepEqual(normalizeDiagnostics(sharedRepaired.diagnostics), baseline);
          const sharedRepairedSecond = await metrics.measure("sharedRepairedSecondApp", () =>
            waitForDiagnostics(session, secondLeafUri, 1, false),
          );
          assert.deepEqual(normalizeDiagnostics(sharedRepairedSecond.diagnostics), secondBaseline);

          // A fresh-version no-op edit cannot match any stale publish, so it
          // proves the second app converged after the whole shared cycle.
          const secondWarm = await changeVue(session, metrics, {
            name: "warmNoopSecondApp",
            uri: secondLeafUri,
            version: 2,
            source: secondLeafSource,
          });
          assert.deepEqual(normalizeDiagnostics(secondWarm.diagnostics), secondBaseline);

          // Fire four same-document changes without waiting between them. The
          // server may cancel superseded work, but every publish that escapes
          // must match the content for its own version and the window must end
          // on the repaired version. Keeping the second app open makes this a
          // cancellation oracle inside the real cross-package monorepo session.
          const cancellationStart = publishes.length;
          const expectedByVersion = new Map<number, string[]>();
          let firstLeafVersion = 4;
          for (const source of [leafBrokenSource, firstLeafSource, leafBrokenSource]) {
            firstLeafVersion += 1;
            expectedByVersion.set(
              firstLeafVersion,
              source === firstLeafSource ? baseline : leafBrokenBaseline,
            );
            session.notify("textDocument/didChange", {
              textDocument: { uri: firstLeafUri, version: firstLeafVersion },
              contentChanges: [{ text: source }],
            });
          }
          firstLeafVersion += 1;
          expectedByVersion.set(firstLeafVersion, baseline);
          const cancellationConverged = await metrics.measure("cancellationConverge", () => {
            session.notify("textDocument/didChange", {
              textDocument: { uri: firstLeafUri, version: firstLeafVersion },
              contentChanges: [{ text: firstLeafSource }],
            });
            return waitForDiagnostics(session, firstLeafUri, firstLeafVersion, false);
          });
          assert.deepEqual(normalizeDiagnostics(cancellationConverged.diagnostics), baseline);
          assertCancellationWindow(
            publishes.slice(cancellationStart),
            firstLeafUri,
            expectedByVersion,
            firstLeafVersion,
          );

          session.notify("textDocument/didClose", { textDocument: { uri: firstLeafUri } });
          session.notify("textDocument/didClose", { textDocument: { uri: secondLeafUri } });
          session.notify("textDocument/didClose", { textDocument: { uri: sharedUri } });
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
            fixture: "vue-vben-admin/apps+packages+playground",
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

// The pinned fixture is hydrated without a pnpm install, so `vue` type
// declarations are unavailable and `defineOptions`/`withDefaults` cannot be
// resolved. The patch keeps both views on the props-typed SFC shape the
// Misskey oracle exercises while wiring them to the shared `packages/`
// component through an explicit cross-package import.
function prepareLeaf(source: string): string {
  const patched = replaceExactly(
    source,
    "import { Fallback } from '@vben/common-ui';\n\ndefineOptions({ name: 'FallbackOfflineDemo' });",
    `import VbenCopyright from '${sharedComponentImport}';\n\nconst ${symbol}: number = 1;\n\ndefineProps<{ mirrorTone?: string }>();`,
  );
  return replaceExactly(
    patched,
    '<Fallback status="offline" />',
    `<div>\n    <VbenCopyright :date="String(${symbol})" />\n    <span>{{ ${symbol} }}</span>\n  </div>`,
  );
}

function prepareSharedComponent(source: string): string {
  const patched = replaceExactly(source, "defineOptions({\n  name: 'Copyright',\n});\n\n", "");
  return replaceExactly(
    patched,
    "withDefaults(defineProps<Props>(), {\n  companyName: 'Vben Admin',\n  companySiteLink: '',\n  date: '2024',\n  icp: '',\n  icpLink: '',\n});",
    "defineProps<Props>();",
  );
}
