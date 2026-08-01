import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { isDiagnosticsForUri } from "../../tooling/support/lsp/assertions.ts";
import type { LspSession } from "../../tooling/support/lsp/session.ts";
import type { ChurnMetrics } from "./churn-metrics.ts";
import type { PublishRecord } from "./churn-oracle.ts";
import {
  assertSingleInjectedMismatch,
  diagnosticsTimeoutMs,
  normalizeDiagnostics,
  waitForDiagnostics,
} from "./lsp-oracle.ts";

type WorkspaceSymbol = { name: string; location: { uri: string } };

const fileName = "__vize_ephemeral_lifecycle.vue";
const symbol = "createdLifecycleProbe";
const source = `<script setup lang="ts">
const ${symbol}: number = 'broken';
void ${symbol};
</script>
`;

/**
 * Repeatedly creates and deletes one real SFC while the same server stays
 * alive. Exact diagnostic/clear publishes are the lifecycle fences; workspace
 * symbols prove the open-document index neither misses nor retains an epoch.
 */
export async function runFileLifecycleChurn(options: {
  session: LspSession;
  metrics: ChurnMetrics;
  workspaceDir: string;
  cycles: number;
  publishes: PublishRecord[];
}): Promise<void> {
  const { session, metrics, workspaceDir, cycles, publishes } = options;
  const filePath = path.join(workspaceDir, "src", fileName);
  const uri = pathToFileURL(filePath).href;
  const streamStart = publishes.length;
  let referencePayload: string[] | null = null;

  assert.equal(fs.existsSync(filePath), false, `${filePath} must start absent`);
  try {
    for (let cycle = 0; cycle < cycles; cycle += 1) {
      const version = cycle + 1;
      await metrics.measure("fileLifecycle", async () => {
        assert.equal(fs.existsSync(filePath), false, `cycle ${cycle}: stale file remained`);
        fs.writeFileSync(filePath, source, { encoding: "utf8", flag: "wx" });
        assert.equal(fs.readFileSync(filePath, "utf8"), source, `cycle ${cycle}: bytes changed`);

        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version, text: source },
        });
        const opened = await waitForDiagnostics(session, uri, version, true);
        assertSingleInjectedMismatch(opened.diagnostics, [], source, symbol);
        const payload = normalizeDiagnostics(opened.diagnostics);
        if (referencePayload == null) referencePayload = payload;
        else assert.deepEqual(payload, referencePayload, `cycle ${cycle}: diagnostics changed`);
        await assertIndexedExactlyOnce(session, uri);

        session.notify("textDocument/didClose", { textDocument: { uri } });
        await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) =>
            isDiagnosticsForUri(params, uri) &&
            params.version == null &&
            params.diagnostics.length === 0,
          diagnosticsTimeoutMs,
        );
        fs.unlinkSync(filePath);
        assert.equal(fs.existsSync(filePath), false, `cycle ${cycle}: delete failed`);
        assert.equal(await session.request("workspace/symbol", { query: symbol }), null);
        assert.equal(
          await session.request("textDocument/documentSymbol", { textDocument: { uri } }),
          null,
        );
      });
      metrics.sampleRss(`file-lifecycle-${cycle}`);
    }

    assert.ok(referencePayload, "file lifecycle must run at least once");
    assertLifecycleStream(publishes.slice(streamStart), uri, cycles, referencePayload);
  } finally {
    if (fs.existsSync(filePath)) fs.unlinkSync(filePath);
  }
}

async function assertIndexedExactlyOnce(session: LspSession, uri: string): Promise<void> {
  const symbols = (await session.request("workspace/symbol", { query: symbol })) as
    | WorkspaceSymbol[]
    | null;
  assert.ok(Array.isArray(symbols), JSON.stringify(symbols));
  assert.deepEqual(
    symbols.map((entry) => ({ name: entry.name, uri: entry.location.uri })),
    [{ name: symbol, uri }],
  );
}

function assertLifecycleStream(
  records: PublishRecord[],
  uri: string,
  cycles: number,
  payload: string[],
): void {
  assert.equal(records.length, cycles * 2, "each file epoch must publish diagnostics then clear");
  for (let cycle = 0; cycle < cycles; cycle += 1) {
    assert.deepEqual(records[cycle * 2], {
      uri,
      version: cycle + 1,
      payload,
      mismatch: true,
    });
    assert.deepEqual(records[cycle * 2 + 1], {
      uri,
      version: null,
      payload: [],
      mismatch: false,
    });
  }
}
