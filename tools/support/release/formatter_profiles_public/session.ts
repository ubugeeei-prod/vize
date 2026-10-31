import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { InstalledAliasSession } from "../../../../tests/tooling/support/lsp/installed-alias-replay/protocol.ts";
import type { JsonRecord } from "../../json-value.ts";
import { inventory, type Entry } from "../editor_vite_stub/filesystem.ts";
import { hook, observed, verifyJournal } from "../editor_vite_stub/custody.ts";
import { resetAuthoredCase, type Context } from "./install.ts";

export const formattingInitialization = {
  editor: true,
  lint: false,
  typecheck: false,
  formatting: true,
} as const;
export type FormattingSession = {
  uri: string;
  request(method: string, params: JsonRecord, expected: unknown): Promise<JsonRecord>;
  change(text: string, version: number): void;
};
export function assertWholeReply(packet: JsonRecord, id: number, expected: unknown): void {
  assert.deepEqual(packet, { jsonrpc: "2.0", id, result: expected });
}
export function assertWholeFilesystem(actual: Entry[], expected: Entry[], phase: string): void {
  assert.deepEqual(
    actual,
    expected,
    "whole authored project and external installation must stay exact: " + phase,
  );
}

/** Original authored cwd/rootUri and strict whole project/install vectors. */
export async function withFormattingSession(
  context: Context,
  label: string,
  source: string,
  configuration: Buffer,
  action: (session: FormattingSession) => Promise<void>,
): Promise<void> {
  resetAuthoredCase(context, label);
  writeFileSync(join(context.root, "App.vue"), source);
  writeFileSync(join(context.root, "vite.config.mjs"), configuration);
  const baseline = inventory(context.root);
  context.record.json(label + ".whole-before.json", baseline);
  let guardOrdinal = 0;
  const guard = (phase: string) => {
    const actual = inventory(context.root);
    context.record.json(label + ".whole-" + guardOrdinal++ + "-" + phase + ".json", actual);
    const changed = actual.filter(
      (row) =>
        row.kind === "file" &&
        !baseline.some((old) => old.path === row.path && old.sha256 === row.sha256),
    );
    const text: unknown[] = [];
    for (const row of changed) {
      if (
        (row.bytes ?? Infinity) > 2 * 1024 * 1024 ||
        !/\.(?:json|log|[cm]?[jt]sx?|map|toml|ya?ml)$/u.test(row.path)
      )
        continue;
      const bytes = readFileSync(join(context.root, row.path));
      try {
        const decoded = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
        if (!decoded.includes("\0"))
          text.push({ path: row.path, rawBase64: bytes.toString("base64") });
      } catch {
        /* Keep whole inventory hashes; binary images are never uploaded. */
      }
    }
    context.record.json(label + ".changed-text-" + phase + ".json", text);
    assertWholeFilesystem(actual, baseline, phase);
  };
  const custody = observed(context.root, context.identity, context.record, label);
  let session: InstalledAliasSession | undefined,
    failed = false,
    firstFailure: unknown;
  const retain = (error: unknown) => {
    if (!failed) {
      failed = true;
      firstFailure = error;
    }
  };
  const replies: unknown[] = [];
  try {
    session = await InstalledAliasSession.launch({
      nodePath: process.execPath,
      cliPath: context.cli,
      custodyHookPath: hook,
      projectRoot: context.root,
      env: custody.env,
      outputRoot: join(context.record.root, "lsp", label),
      timeoutMs: 120_000,
      crossFile: false,
      initializationOptions: formattingInitialization,
    });
    guard("initialized");
    const active = session;
    const uri = pathToFileURL(join(context.root, "App.vue")).href;
    active.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    let requestId = 2;
    await action({
      uri,
      async request(method, params, expected) {
        const id = requestId++;
        let response: JsonRecord | undefined,
          error: unknown,
          rejected = false;
        try {
          response = await active.request(method, params);
          replies.push({ method, params, expected, actual: response });
          context.record.json(label + ".replies.json", replies);
        } catch (failure) {
          rejected = true;
          error = failure;
        }
        try {
          guard("request-" + id);
        } catch (failure) {
          if (!rejected) {
            rejected = true;
            error = failure;
          }
        }
        if (rejected) throw error;
        assert(response);
        assertWholeReply(response, id, expected);
        return response;
      },
      change(text, version) {
        active.notify("textDocument/didChange", {
          textDocument: { uri, version },
          contentChanges: [{ text }],
        });
      },
    });
  } catch (error) {
    retain(error);
  } finally {
    if (session) {
      try {
        const receipt = await session.shutdown();
        context.record.json(label + ".lsp-terminal.json", receipt);
        guard("shutdown");
        assert.equal(receipt.closed, true);
        assert.equal(receipt.exitCode, 0);
        assert.equal(receipt.signal, null);
        assert.equal(receipt.remainingFrameBytes, 0);
        assert.deepEqual(receipt.failures, []);
        assert.equal(receipt.parsingFailed, false);
        verifyJournal(custody.journal, context.identity, 0);
      } catch (error) {
        retain(error);
      }
    } else {
      try {
        guard("failed-startup");
      } catch (error) {
        retain(error);
      }
    }
  }
  if (failed) throw firstFailure;
}
