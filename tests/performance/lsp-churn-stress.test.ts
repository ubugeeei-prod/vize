import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { withPinnedFixtureWorkspace } from "../_helpers/realworld-patch.ts";
import { resolveTsgoBinary } from "../_helpers/realworld-typecheck.ts";
import { isDiagnosticsForUri } from "../tooling/support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "../tooling/support/lsp/protocol.ts";
import { LspSession } from "../tooling/support/lsp/session.ts";
import { ChurnMetrics } from "./support/churn-metrics.ts";
import {
  assertCancellationWindow,
  assertDependencyBaseline,
  assertSameStream,
  assertStreamOrdering,
  recordPublishes,
} from "./support/churn-oracle.ts";
import { countFiles } from "./support/incremental-metrics.ts";
import {
  assertSingleInjectedMismatch,
  diagnosticsTimeoutMs,
  normalizeDiagnostics,
  replaceExactly,
  waitForDiagnostics,
} from "./support/lsp-oracle.ts";

// Same production pair as the Misskey incremental oracle: a leaf component
// plus the shared component it imports, so churn covers both same-document
// recomputes and cross-document dependency invalidation.
const componentPath = "packages/frontend/src/components/MkDivider.vue";
const dependencyPath = "packages/frontend/src/components/MkCodeInline.vue";
const symbol = "churnMirror";

// One churn cycle = leaf broken -> leaf repaired -> shared broken -> shared
// repaired, awaiting the leaf's exact republish each time. The registry block
// pins cyclesPerPhase; the identical cycle sequence runs twice in one server
// session (phase A, then phase B) so nondeterministic diagnostics, stale
// overlays, or churn-driven decay diverge the phases and fail hard.
test(
  "Misskey LSP edit churn stays deterministic, converged, and leak-free",
  { timeout: 300_000 },
  async () => {
    // Fail closed like the VIZE_TEST_REQUIRE_TSGO lanes: without the
    // Corsa/tsgo backend the server publishes no type diagnostics and the
    // churn oracle would be meaningless.
    resolveTsgoBinary();
    await withPinnedFixtureWorkspace(
      { fixtureId: "misskey", includePaths: ["packages/frontend"] },
      async (fixture) => {
        const workspaceDir = path.join(fixture.workspaceDir, "packages/frontend");
        const sourceDir = path.join(workspaceDir, "src");
        const componentUri = pathToFileURL(fixture.resolve(componentPath)).href;
        const dependencyUri = pathToFileURL(fixture.resolve(dependencyPath)).href;
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
        const metrics = new ChurnMetrics(session.processId, {
          id: "misskey-lsp-churn",
          title: "Misskey LSP Churn Stress Oracle",
        });
        const cycles = metrics.budget.cyclesPerPhase;
        const publishes = recordPublishes(session);
        let baseline: string[] = [];
        let failure: unknown;

        let leafVersion = 1;
        let dependencyVersion = 1;
        const editLeaf = (source: string, expectError: boolean) => {
          leafVersion += 1;
          session.notify("textDocument/didChange", {
            textDocument: { uri: componentUri, version: leafVersion },
            contentChanges: [{ text: source }],
          });
          return waitForDiagnostics(session, componentUri, leafVersion, expectError);
        };
        const editShared = (source: string, expectError: boolean) => {
          dependencyVersion += 1;
          session.notify("textDocument/didChange", {
            textDocument: { uri: dependencyUri, version: dependencyVersion },
            contentChanges: [{ text: source }],
          });
          return waitForDiagnostics(session, componentUri, leafVersion, expectError);
        };
        // A fresh-version no-op edit whose publish is causally behind every
        // publish of the finished phase, so the phase slice is complete and
        // convergence to the clean baseline is proven (no stale publish).
        const fencePhase = async (): Promise<number> => {
          const publish = await metrics.measure("phaseFence", () => editLeaf(cleanSource, false));
          assert.deepEqual(normalizeDiagnostics(publish.diagnostics), baseline);
          const fenceIndex = publishes.findIndex(
            (record) => record.uri === componentUri && record.version === leafVersion,
          );
          assert.ok(fenceIndex >= 0, "the phase fence publish must be recorded");
          return fenceIndex;
        };
        // The reference cycle payload sequence: pinned exactly on the first
        // cycle, then every later cycle in both phases must reproduce it.
        let reference: string[][] | null = null;
        const runCycle = async (label: string) => {
          const consumed: PublishDiagnosticsParams[] = await metrics.measure("cycle", async () => [
            await editLeaf(leafBrokenSource, true),
            await editLeaf(cleanSource, false),
            await editShared(brokenDependency, true),
            await editShared(cleanDependency, false),
          ]);
          metrics.sampleRss(label);
          const payloads = consumed.map((publish) => normalizeDiagnostics(publish.diagnostics));
          if (reference == null) {
            assertSingleInjectedMismatch(
              consumed[0].diagnostics,
              baseline,
              leafBrokenSource,
              symbol,
            );
            assertSingleInjectedMismatch(consumed[2].diagnostics, baseline, cleanSource, symbol, {
              attributeName: "code",
            });
            assert.deepEqual(payloads[1], baseline);
            assert.deepEqual(payloads[3], baseline);
            reference = payloads;
          } else {
            assert.deepEqual(payloads, reference, `cycle ${label} diverged from the first cycle`);
          }
        };

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
              textDocument: { uri: componentUri, languageId: "vue", version: 1, text: cleanSource },
            });
            return waitForDiagnostics(session, componentUri, 1);
          });
          assert.equal(cleanPublish.diagnostics.length, 0, JSON.stringify(cleanPublish));
          baseline = normalizeDiagnostics(cleanPublish.diagnostics);
          const dependencyPublish = await waitForDiagnostics(session, dependencyUri, 1);
          assertDependencyBaseline(dependencyPublish.diagnostics, "dependency baseline");
          assert.equal(publishes.length, 2, "cold open must publish each document exactly once");
          metrics.sampleRss("coldOpen");

          const phaseStart: number[] = [];
          const phaseEnd: number[] = [];
          for (const phase of ["A", "B"]) {
            phaseStart.push(publishes.length);
            for (let cycle = 0; cycle < cycles; cycle += 1) {
              await runCycle(`${phase}${cycle}`);
            }
            phaseEnd.push(await fencePhase());
          }
          const sliceA = publishes.slice(phaseStart[0], phaseEnd[0]);
          const sliceB = publishes.slice(phaseStart[1], phaseEnd[1]);
          assert.equal(
            sliceA.length,
            cycles * 6,
            "each cycle must publish exactly 6 times: leaf broken, leaf repaired, and both documents on each shared edit",
          );
          assertSameStream(sliceB, sliceA, "phase B vs phase A");

          // Supersession: rapid successive didChange with no waits between.
          // The server may skip superseded intermediate publishes, but must
          // converge on the final version's clean diagnostics.
          const windowStart = publishes.length;
          const expectedByVersion = new Map<number, string[]>();
          for (const source of [leafBrokenSource, cleanSource, leafBrokenSource]) {
            leafVersion += 1;
            expectedByVersion.set(leafVersion, source === cleanSource ? baseline : reference![0]);
            session.notify("textDocument/didChange", {
              textDocument: { uri: componentUri, version: leafVersion },
              contentChanges: [{ text: source }],
            });
          }
          leafVersion += 1;
          expectedByVersion.set(leafVersion, baseline);
          const converged = await metrics.measure("cancellationConverge", () => {
            session.notify("textDocument/didChange", {
              textDocument: { uri: componentUri, version: leafVersion },
              contentChanges: [{ text: cleanSource }],
            });
            return waitForDiagnostics(session, componentUri, leafVersion, false);
          });
          assert.deepEqual(normalizeDiagnostics(converged.diagnostics), baseline);
          metrics.sampleRss("cancellation");

          session.notify("textDocument/didClose", { textDocument: { uri: componentUri } });
          session.notify("textDocument/didClose", { textDocument: { uri: dependencyUri } });
          await metrics.measure("closeClear", async () => {
            for (const uri of [componentUri, dependencyUri]) {
              await session.waitForNotification(
                "textDocument/publishDiagnostics",
                (params) =>
                  isDiagnosticsForUri(params, uri) &&
                  params.version == null &&
                  params.diagnostics.length === 0,
                diagnosticsTimeoutMs,
              );
            }
          });
          metrics.sampleRss("closed");

          assertStreamOrdering(publishes, [componentUri, dependencyUri]);
          const firstVersionless = publishes.findIndex((record) => record.version == null);
          assertCancellationWindow(
            publishes.slice(windowStart, firstVersionless),
            componentUri,
            expectedByVersion,
            leafVersion,
          );
        } catch (error) {
          failure = error;
          // A hang or divergence is only diagnosable from the publish stream
          // and the server log, so surface both in the test output.
          console.error(
            `churn failed at leaf v${leafVersion}, dependency v${dependencyVersion}; ` +
              `observed ${publishes.length} publishes, tail: ${JSON.stringify(
                publishes.slice(-8).map((record) => ({
                  uri: record.uri.split("/").pop(),
                  version: record.version,
                  mismatch: record.mismatch,
                  diagnostics: record.payload.length,
                })),
              )}\nserver stderr tail: ${JSON.stringify(session.stderrText.slice(-2000))}`,
          );
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
            publishes: publishes.length,
          },
          failure,
        );
        if (failure != null) throw failure;
      },
    );
  },
);

function prepareComponent(source: string): string {
  const patched = replaceExactly(
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
