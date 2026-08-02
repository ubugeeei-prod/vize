import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { LspDiagnostic, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const malformedSource = "<template><div></div>";
const repairedSource = "<template><div></div></template>";
const malformedDiagnostics: LspDiagnostic[] = [
  {
    message: "Malformed <template> block: the closing tag is missing.",
    range: {
      start: { line: 0, character: 0 },
      end: { line: 0, character: malformedSource.length },
    },
    severity: 1,
    source: "vize/sfc",
  },
];

async function openAndExpectMalformed(
  session: LspSession,
  uri: string,
  version: number,
): Promise<PublishDiagnosticsParams> {
  session.notify("textDocument/didOpen", {
    textDocument: {
      uri,
      languageId: "vue",
      version,
      text: malformedSource,
    },
  });

  const publish = (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri) && params.version === version,
  )) as PublishDiagnosticsParams;
  assert.equal(publish.version, version);
  assert.deepEqual(publish.diagnostics, malformedDiagnostics);
  return publish;
}

test("vize lsp revalidates one malformed document after graceful and forced server restarts", async (t) => {
  const restartCases = [
    {
      label: "shutdown request followed by exit",
      stop: (session: LspSession) => session.shutdown(),
    },
    {
      label: "forced process kill",
      stop: (session: LspSession) => session.kill("SIGKILL"),
    },
  ] as const;

  for (const restartCase of restartCases) {
    await t.test(restartCase.label, async () => {
      const testRootDir = path.join(
        testOutputRoot,
        `lsp-server-restart-${restartCase.label.replaceAll(" ", "-")}`,
      );
      fs.mkdirSync(testRootDir, { recursive: true });
      const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
      const filePath = path.join(workspaceDir, "Restart.vue");
      const uri = pathToFileURL(filePath).href;
      fs.writeFileSync(filePath, malformedSource, "utf8");

      let firstSession: LspSession | undefined = new LspSession();
      let restartedSession: LspSession | undefined;

      try {
        await firstSession.initialize(workspaceDir, {
          editor: true,
          lint: false,
          typecheck: false,
        });
        const firstProcessId = firstSession.processId;
        await openAndExpectMalformed(firstSession, uri, 41);

        await restartCase.stop(firstSession);
        firstSession = undefined;

        restartedSession = new LspSession();
        assert.notEqual(
          restartedSession.processId,
          firstProcessId,
          "restart must launch a fresh vize lsp process",
        );
        await restartedSession.initialize(workspaceDir, {
          editor: true,
          lint: false,
          typecheck: false,
        });

        const restartedPublishes: PublishDiagnosticsParams[] = [];
        restartedSession.notificationObservers.push((method, params) => {
          if (method === "textDocument/publishDiagnostics" && isDiagnosticsForUri(params, uri)) {
            restartedPublishes.push(params);
          }
        });

        await openAndExpectMalformed(restartedSession, uri, 1);

        restartedSession.notify("textDocument/didChange", {
          textDocument: { uri, version: 2 },
          contentChanges: [{ text: repairedSource }],
        });
        const repairedPublish = (await restartedSession.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri) && params.version === 2,
        )) as PublishDiagnosticsParams;
        assert.deepEqual(repairedPublish.diagnostics, []);

        restartedSession.notify("textDocument/didChange", {
          textDocument: { uri, version: 3 },
          contentChanges: [{ text: repairedSource }],
        });
        const stablePublish = (await restartedSession.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri) && params.version === 3,
        )) as PublishDiagnosticsParams;
        assert.deepEqual(stablePublish.diagnostics, []);

        assert.deepEqual(
          restartedPublishes.map(({ version, diagnostics }) => ({ version, diagnostics })),
          [
            { version: 1, diagnostics: malformedDiagnostics },
            { version: 2, diagnostics: [] },
            { version: 3, diagnostics: [] },
          ],
          "a fresh server must not publish stale versions or diagnostics after repair",
        );
      } finally {
        if (restartedSession) {
          await restartedSession.shutdown();
        }
        if (firstSession) {
          await firstSession.shutdown();
        }
        fs.rmSync(workspaceDir, { recursive: true, force: true });
        fs.rmSync(testRootDir, { recursive: true, force: true });
      }
    });
  }
});
