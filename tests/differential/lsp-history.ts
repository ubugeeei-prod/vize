// LSP fix-history response sessions (#6883).
//
// Each case is one historical maestro fix replayed as a scripted stdio session
// against the source-built `vize lsp`: a workspace of authored files, the
// initialization options, and ordered steps (open, change, disk writes, watched
// file events, requests and settled diagnostics). Every recording step's
// complete JSON-RPC result is pinned in `expected.json`; only the temporary
// workspace location and the platform TypeScript package name are normalized.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { isDeepStrictEqual } from "node:util";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { sha256 } from "./sha256.ts";
import { LspWire } from "./lsp-wire.ts";
import { INITIALIZE_CAPABILITIES } from "./lsp-manifest.ts";
import { sourceLspRuntimeEnvironment } from "../tooling/support/lsp/runtime-environment.ts";
import type { JsonRpcMessage } from "./lsp-types.ts";
import {
  applyChange,
  languageId,
  materialize,
  normalize,
  pickLabels,
  positionOf,
} from "./lsp-history-values.ts";
import type { LspRange } from "./lsp-history-values.ts";

export { applyChange, normalize, pickLabels, positionOf };

export const HISTORY_ROOT = "tests/_fixtures/differential/lsp-history";
export const HISTORY_MANIFEST = `${HISTORY_ROOT}/manifest.json`;
export const HISTORY_LEDGER = "docs/davinci/plan/lsp-fix-history.tsv";
const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));

export type HistoryStep =
  | { open: string; languageId?: string; version?: number; text?: string }
  | { close: string }
  | { change: string; version: number; contentChanges: unknown[] }
  | { save: string }
  | { write: string; text: string }
  | { remove: string }
  | { notify: string; params?: unknown }
  | { closeStderr: true }
  // Records the params of every server-to-client request with this method so far.
  | { serverRequests: string }
  | { diagnostics: string; version?: number | null }
  | {
      request: string;
      uri?: string;
      params?: Record<string, unknown>;
      // `at` resolves the first (or nth) occurrence of `find` plus `delta`
      // UTF-16 units in the current text of `uri` into `params.position`.
      at?: { find: string; delta?: number; nth?: number };
      // Completion lists are large; `labels` keeps the complete items with
      // these labels (in server order) plus the total item count.
      labels?: string[];
      // `from` copies one raw item of an earlier request's result into
      // params[as] (or replaces params when `as` is "$"), e.g. call hierarchy.
      from?: { step: number; label?: string; index?: number; field?: string; as: string };
      retry?: boolean;
    };

export type HistoryCase = {
  version: 1;
  id: string;
  commit: string;
  requirement: string;
  initializationOptions?: Record<string, unknown>;
  capabilities?: Record<string, unknown>;
  recordInitialize?: boolean;
  // "absent" starts the server without the checkout's TypeScript runtime.
  typecheckRuntime?: "absent";
  // Packages linked from the checkout's node_modules (e.g. "vue" also links "@vue").
  packages?: string[];
  // Exit must terminate the server while stdin stays open (no EOF).
  exitWithoutEof?: boolean;
  files: Record<string, string>;
  steps: HistoryStep[];
};
export type HistoryRecord =
  | { step: number; result: unknown }
  | { step: number; error: unknown }
  | { step: number; version: number | null; diagnostics: unknown[] }
  | { step: "initialize"; capabilities: unknown }
  | { step: number; serverRequests: unknown[] };
export type HistoryManifest = {
  schema: "vize.lsp.fix-history.sessions";
  version: 1;
  cases: { id: string; commit: string; case: string; sha256: string; expectedSha256: string }[];
};

export const DEFAULT_OPTIONS = { editor: true, lint: false, typecheck: false };
const ID = /^lsp\/fix-history\/[a-z0-9]+(?:-[a-z0-9]+)*$/;

export const caseDirectory = (id: string) => id.slice("lsp/fix-history/".length);

export function validateHistoryCase(value: HistoryCase): HistoryCase {
  assert.equal(value.version, 1);
  assert.match(value.id, ID, "case id");
  assert.match(value.commit, /^[a-f0-9]{40}$/, `${value.id}: commit`);
  assert(typeof value.requirement === "string" && value.requirement.length > 0);
  const names = Object.keys(value.files);
  assert(names.length > 0, `${value.id}: files`);
  for (const name of names) {
    assert(!path.isAbsolute(name) && !name.includes("\\"), `${value.id}: ${name}`);
    assert(name.split("/").every((part) => part !== "" && part !== "." && part !== ".."));
    assert.equal(typeof value.files[name], "string");
  }
  assert(value.steps.length > 0, `${value.id}: steps`);
  assert(
    value.steps.some(
      (step) => "request" in step || "diagnostics" in step || "serverRequests" in step,
    ) || value.recordInitialize,
    `${value.id}: at least one recorded response`,
  );
  return value;
}

export function loadHistoryManifest(root = repositoryRoot) {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(root, HISTORY_MANIFEST), "utf8"),
  ) as HistoryManifest;
  assert.equal(manifest.schema, "vize.lsp.fix-history.sessions");
  assert.equal(manifest.version, 1);
  const ids = manifest.cases.map((entry) => entry.id);
  assert.deepEqual(ids, [...ids].sort(), "manifest cases stay sorted by id");
  assert.equal(new Set(ids).size, ids.length, "duplicate case id");
  return {
    manifest,
    cases: manifest.cases.map((entry) => {
      const caseBytes = fs.readFileSync(path.join(root, HISTORY_ROOT, entry.case));
      const expectedPath = path.join(root, HISTORY_ROOT, caseDirectory(entry.id), "expected.json");
      const expectedBytes = fs.readFileSync(expectedPath);
      assert.equal(sha256(caseBytes), entry.sha256, `${entry.id}: case SHA256 mismatch`);
      assert.equal(sha256(expectedBytes), entry.expectedSha256, `${entry.id}: SHA256 mismatch`);
      const data = validateHistoryCase(JSON.parse(caseBytes.toString("utf8")) as HistoryCase);
      assert.equal(data.id, entry.id);
      assert.equal(data.commit, entry.commit);
      assert.equal(entry.case, `${caseDirectory(entry.id)}/case.json`);
      const expected = JSON.parse(expectedBytes.toString("utf8")) as HistoryRecord[];
      assert.equal(expected.length, recordCount(data), `${entry.id}: one record per step`);
      return { data, expected };
    }),
  };
}

function recordCount(data: HistoryCase) {
  return (
    (data.recordInitialize ? 1 : 0) +
    data.steps.filter(
      (step) => "request" in step || "diagnostics" in step || "serverRequests" in step,
    ).length
  );
}

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

type Mode = { capture: true } | { capture: false; expected: HistoryRecord[] };

async function settledDiagnostics(
  wire: LspWire,
  uri: string,
  version: number | null | undefined,
  mode: Mode,
  expected: HistoryRecord | undefined,
  workspace: string,
): Promise<{ version: number | null; diagnostics: unknown[] }> {
  const matches = (message: JsonRpcMessage) =>
    message.method === "textDocument/publishDiagnostics" &&
    message.params?.uri === uri &&
    (version === undefined || (message.params?.version ?? null) === version);
  const latest = () => {
    const found = wire.messages.filter(matches).at(-1);
    return found
      ? {
          version: (found.params?.version as number | undefined) ?? null,
          diagnostics: found.params?.diagnostics as unknown[],
        }
      : undefined;
  };
  await wire.waitFor(matches, 60_000);
  if (mode.capture) {
    // Capture only a quiescent state: no newer publication for three seconds.
    let count = wire.messages.filter(matches).length;
    for (;;) {
      await delay(3_000);
      const next = wire.messages.filter(matches).length;
      if (next === count) break;
      count = next;
    }
    return latest()!;
  }
  const want = expected && "diagnostics" in expected ? expected : undefined;
  const deadline = Date.now() + 60_000;
  for (;;) {
    const current = latest()!;
    const shaped = normalize(current, workspace);
    if (want && isDeepStrictEqual(shaped, { version: want.version, diagnostics: want.diagnostics }))
      return current;
    if (Date.now() > deadline) return current;
    await delay(100);
  }
}

/** Run one case and return its normalized records. */
export async function runHistoryCase(
  data: HistoryCase,
  binaryPath: string,
  mode: Mode,
): Promise<{ records: HistoryRecord[]; stderr: string }> {
  const workspace = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-history-")));
  let wire: LspWire | undefined;
  const records: HistoryRecord[] = [];
  try {
    for (const [name, text] of Object.entries(data.files)) {
      const target = path.join(workspace, name);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, text);
    }
    for (const name of data.packages ?? []) {
      const resolved = path.dirname(
        createRequire(path.join(repositoryRoot, "tests/package.json")).resolve(
          `${name}/package.json`,
        ),
      );
      const linkType = process.platform === "win32" ? "junction" : "dir";
      fs.mkdirSync(path.join(workspace, "node_modules"), { recursive: true });
      fs.symlinkSync(resolved, path.join(workspace, "node_modules", name), linkType);
      if (name === "vue") {
        fs.symlinkSync(
          path.join(path.dirname(resolved), "@vue"),
          path.join(workspace, "node_modules/@vue"),
          linkType,
        );
      }
    }
    const workspaceUri = pathToFileURL(workspace).href;
    const uriOf = (file: string) => pathToFileURL(path.join(workspace, file)).href;
    const environment =
      data.typecheckRuntime === "absent"
        ? Object.fromEntries(
            Object.entries(process.env).filter(
              ([name]) => !/^(?:CORSA|TSGO|TSRS)_(?:PATH|EXECUTABLE)$/.test(name),
            ),
          )
        : sourceLspRuntimeEnvironment(repositoryRoot);
    wire = new LspWire(binaryPath, ["lsp"], workspace, environment);
    // Answer server-to-client requests (registerCapability, configuration...)
    // like an editor would, so the server never waits on the client.
    const active = wire;
    const receive = active.receive.bind(active);
    active.receive = (message: JsonRpcMessage) => {
      receive(message);
      if (message.id !== undefined && typeof message.method === "string")
        active.send({ jsonrpc: "2.0", id: message.id, result: null });
    };
    const initialized = await wire.request("initialize", {
      processId: process.pid,
      rootUri: workspaceUri,
      capabilities: data.capabilities ?? INITIALIZE_CAPABILITIES,
      initializationOptions: data.initializationOptions ?? DEFAULT_OPTIONS,
      workspaceFolders: [{ uri: workspaceUri, name: path.basename(workspace) }],
    });
    assert(initialized.result, `initialize failed: ${JSON.stringify(initialized.error)}`);
    if (data.recordInitialize) {
      const capabilities = (initialized.result as { capabilities: unknown }).capabilities;
      records.push({ step: "initialize", capabilities: normalize(capabilities, workspace) });
    }
    wire.notify("initialized", {});
    const versions = new Map<string, number>();
    const texts = new Map<string, string>(Object.entries(data.files));
    const raw = new Map<number, unknown>();
    for (const [index, step] of data.steps.entries()) {
      if ("open" in step) {
        const version = step.version ?? 1;
        versions.set(step.open, version);
        if (step.text !== undefined) texts.set(step.open, step.text);
        wire.notify("textDocument/didOpen", {
          textDocument: {
            uri: uriOf(step.open),
            languageId: step.languageId ?? languageId(step.open),
            version,
            text: step.text ?? fs.readFileSync(path.join(workspace, step.open), "utf8"),
          },
        });
      } else if ("close" in step) {
        wire.notify("textDocument/didClose", { textDocument: { uri: uriOf(step.close) } });
      } else if ("change" in step) {
        versions.set(step.change, step.version);
        for (const change of step.contentChanges as { range?: LspRange; text: string }[])
          texts.set(step.change, applyChange(texts.get(step.change) ?? "", change));
        wire.notify("textDocument/didChange", {
          textDocument: { uri: uriOf(step.change), version: step.version },
          contentChanges: step.contentChanges,
        });
      } else if ("save" in step) {
        wire.notify("textDocument/didSave", { textDocument: { uri: uriOf(step.save) } });
      } else if ("write" in step) {
        const target = path.join(workspace, step.write);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.writeFileSync(target, step.text);
        texts.set(step.write, step.text);
      } else if ("remove" in step) {
        fs.rmSync(path.join(workspace, step.remove), { recursive: true, force: true });
      } else if ("serverRequests" in step) {
        const method = step.serverRequests;
        const seen = wire.messages.filter(
          (message) => message.method === method && message.id !== undefined,
        );
        records.push({
          step: index,
          serverRequests: seen.map((message) => normalize(message.params ?? null, workspace)),
        });
      } else if ("closeStderr" in step) {
        wire.child.stderr.destroy();
      } else if ("notify" in step) {
        wire.notify(step.notify, materialize(step.params, workspace) as Record<string, unknown>);
      } else if ("diagnostics" in step) {
        const expected = mode.capture ? undefined : mode.expected[records.length];
        const version =
          step.version === undefined ? (versions.get(step.diagnostics) ?? null) : step.version;
        const settled = await settledDiagnostics(
          wire,
          uriOf(step.diagnostics),
          version === null ? undefined : version,
          mode,
          expected,
          workspace,
        );
        records.push({ step: index, ...(normalize(settled, workspace) as typeof settled) });
      } else {
        let params: Record<string, unknown> = {
          ...(step.uri === undefined ? {} : { textDocument: { uri: uriOf(step.uri) } }),
          ...(materialize(step.params ?? {}, workspace) as Record<string, unknown>),
          ...(step.at ? { position: positionOf(texts.get(step.uri!)!, step.at) } : {}),
        };
        if (step.from) {
          const earlier = raw.get(step.from.step);
          const list = (
            Array.isArray(earlier) ? earlier : ((earlier as { items?: unknown[] })?.items ?? [])
          ) as { label?: string }[];
          const picked =
            step.from.label === undefined
              ? list[step.from.index ?? 0]
              : list.find((candidate) => candidate.label === step.from!.label);
          const item =
            step.from.field === undefined
              ? picked
              : (picked as Record<string, unknown> | undefined)?.[step.from.field];
          assert(item, `step ${index}: no item from step ${step.from.step}`);
          params =
            step.from.as === "$"
              ? (item as Record<string, unknown>)
              : { ...params, [step.from.as]: item };
        }
        const expected = mode.capture ? undefined : mode.expected[records.length];
        let record: HistoryRecord;
        const deadline = Date.now() + 30_000;
        let previous: unknown;
        for (;;) {
          const reply = await wire.request(step.request, params, 60_000);
          raw.set(index, reply.result);
          const result = step.labels ? pickLabels(reply.result, step.labels) : reply.result;
          record = (
            reply.error
              ? { step: index, error: normalize(reply.error, workspace) }
              : { step: index, result: normalize(result ?? null, workspace) }
          ) as HistoryRecord;
          if (!step.retry) break;
          if (
            mode.capture ? isDeepStrictEqual(record, previous) : isDeepStrictEqual(record, expected)
          )
            break;
          if (Date.now() > deadline) break;
          previous = record;
          await delay(mode.capture ? 1_500 : 200);
        }
        records.push(record);
      }
    }
    if (data.exitWithoutEof) {
      const shutdown = await wire.request("shutdown", undefined, 10_000);
      assert.equal(shutdown.result, null, "shutdown must succeed");
      wire.notify("exit");
      let timer: ReturnType<typeof setTimeout> | undefined;
      await Promise.race([
        wire.exited,
        new Promise((_, reject) => {
          timer = setTimeout(() => reject(new Error("exit did not terminate the server")), 5_000);
        }),
      ]).finally(() => clearTimeout(timer));
      assert.equal(wire.exitStatus, 0, "LSP must exit successfully after exit");
    } else {
      await wire.finish();
    }
    return { records, stderr: Buffer.concat(wire.stderrChunks).toString("utf8") };
  } finally {
    if (wire) await wire.stop().catch(() => {});
    fs.rmSync(workspace, { recursive: true, force: true });
  }
}

/** Run every case with bounded concurrency and report mismatches. */
export async function runHistoryPack(binaryPath: string, concurrency = 4, root = repositoryRoot) {
  const { cases } = loadHistoryManifest(root);
  const results: { id: string; equal: boolean; actual?: HistoryRecord[]; error?: string }[] = [];
  let next = 0;
  await Promise.all(
    Array.from({ length: Math.min(concurrency, cases.length) }, async () => {
      while (next < cases.length) {
        const { data, expected } = cases[next++];
        try {
          const { records } = await runHistoryCase(data, binaryPath, {
            capture: false,
            expected,
          });
          results.push({
            id: data.id,
            equal: isDeepStrictEqual(records, expected),
            actual: records,
          });
        } catch (error) {
          results.push({ id: data.id, equal: false, error: String(error) });
        }
      }
    }),
  );
  return results.sort((a, b) => a.id.localeCompare(b.id));
}
