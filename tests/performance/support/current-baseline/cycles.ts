import assert from "node:assert/strict";
import { completionLabels } from "../../../tooling/support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "../../../tooling/support/lsp/protocol.ts";
import type { LspSession } from "../../../tooling/support/lsp/session.ts";
import { positionInsideTemplateSymbol } from "../lsp-oracle.ts";
import { loadLspIncrementalBudget } from "../incremental-metrics.ts";
import { baselineObserver } from "./observer.ts";
import { hash } from "./wire.ts";

export function beginBaselineCycle(label: string): number | undefined {
  const observer = baselineObserver();
  if (!observer) return undefined;
  assert.match(label, /^[AB](?:[0-9]|1[0-9])$/);
  return observer.lengths.client;
}

export async function completeBaselineCycle(
  session: LspSession,
  input: {
    label: string;
    start: number | undefined;
    consumed: PublishDiagnosticsParams[];
    uri: string;
    version: number;
    source: string;
    symbol: string;
  },
): Promise<void> {
  const observer = baselineObserver();
  if (!observer) return;
  assert(input.start !== undefined);
  const end = observer.lengths.client;
  const result = await session.request(
    "textDocument/completion",
    {
      textDocument: { uri: input.uri },
      position: positionInsideTemplateSymbol(input.source, input.symbol, "churnM"),
    },
    loadLspIncrementalBudget("misskey-lsp-incremental").budget.laneBudgetsMs.completion,
    (id) => {
      observer.cycle({
        label: input.label,
        start: input.start,
        end,
        consumed: input.consumed,
        completion: {
          id,
          uri: input.uri,
          version: input.version,
          sourceSha256: hash(input.source),
          requiredSymbol: input.symbol,
        },
      });
      return [];
    },
  );
  assert(
    completionLabels(result as never).includes(input.symbol),
    "whole completion response must include the authored symbol",
  );
}

export async function finishBaselineSession(): Promise<void> {
  await baselineObserver()?.closed;
}
