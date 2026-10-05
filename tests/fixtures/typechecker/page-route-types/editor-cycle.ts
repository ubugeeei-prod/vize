import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";
import type { PinnedFixtureWorkspace } from "../../../_helpers/realworld-patch.ts";
import type { CheckReport } from "../../../_helpers/realworld-typecheck.ts";
import {
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "../../../tooling/support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "../../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../../tooling/support/lsp/session.ts";
import {
  assertCli,
  assertEditor,
  type Config,
  type ErrorSpec,
  PAGE_PATH,
  sourceRange,
} from "./support.ts";

type InvalidPhase = { name: string; source: string; errors: ErrorSpec[] };
type Observe = (
  source: string,
  errors: ErrorSpec[],
  sourcePath?: string,
  reference?: (source: string) => string,
  referenceConfig?: Config,
  productConfig?: Config,
) => { rows: ReturnType<typeof assertCli>; report: CheckReport };

export function editorCycleFor(fixture: PinnedFixtureWorkspace, observe: Observe) {
  return async function editorCycle(
    clean: string,
    invalid: InvalidPhase[],
    sourcePath = PAGE_PATH,
    reference: (source: string) => string = (value) => value,
    hoverNeedle?: string,
  ) {
    const baseline = observe(clean, [], sourcePath, reference);
    const uri = pathToFileURL(fixture.resolve(sourcePath)).href;
    const session = new LspSession();
    let version = 1;
    try {
      await session.initialize(fixture.workspaceDir, {
        completion: true,
        editor: true,
        ecosystem: false,
        hover: true,
        lint: false,
        typecheck: true,
      });
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version, text: clean },
      });

      async function assertPublish(source: string, errors: ErrorSpec[], rows = baseline.rows) {
        const publish = (await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) =>
            isDiagnosticsForUri(params, uri) &&
            params.version === version &&
            params.diagnostics.length === errors.length,
          120_000,
        )) as PublishDiagnosticsParams;
        assertEditor(publish.diagnostics, rows, source, errors);
        if (errors.length === 0 && hoverNeedle) {
          const offset = source.indexOf(hoverNeedle);
          assert.notEqual(offset, -1);
          const hover = (await session.request("textDocument/hover", {
            textDocument: { uri },
            position: offsetToPosition(source, offset + 1),
          })) as { contents?: unknown; range?: unknown } | null;
          assert.match(hoverToText(hover), /\bnumber\b/);
          assert.deepEqual(
            hover?.range,
            sourceRange(source, {
              code: 0,
              needle: hoverNeedle,
              token: hoverNeedle,
            }),
          );
        }
      }

      await assertPublish(clean, []);
      for (const phase of invalid) {
        const broken = observe(phase.source, phase.errors, sourcePath, reference);
        assert.deepEqual(broken.report.programs, baseline.report.programs, phase.name);
        assert.deepEqual(
          {
            ...broken.report,
            errorCount: 0,
            files: broken.report.files.map((file) => ({ ...file, diagnostics: [] })),
          },
          baseline.report,
          `${phase.name}: preserve every non-diagnostic report field and ordered file vector`,
        );
        version += 1;
        session.notify("textDocument/didChange", {
          textDocument: { uri, version },
          contentChanges: [{ text: phase.source }],
        });
        await assertPublish(phase.source, phase.errors, broken.rows);
        const repair = observe(clean, [], sourcePath, reference);
        assert.deepEqual(
          repair.report,
          baseline.report,
          `${phase.name}: complete repaired CLI report`,
        );
        version += 1;
        session.notify("textDocument/didChange", {
          textDocument: { uri, version },
          contentChanges: [{ text: clean }],
        });
        await assertPublish(clean, []);
      }
    } finally {
      await session.shutdown();
    }
  };
}
