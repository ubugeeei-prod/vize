import assert from "node:assert/strict";
import {
  hoverToText,
  offsetToPosition,
} from "../../../../../tests/tooling/support/lsp/assertions.ts";
import type {
  JsonRpcMessage,
  PublishDiagnosticsParams,
} from "../../../../../tests/tooling/support/lsp/protocol.ts";
import {
  LspRequestError,
  type LspSession,
} from "../../../../../tests/tooling/support/lsp/session.ts";
import { asString, at, isRecord } from "../../../json-value.ts";
import type { JsonRecord } from "../../../json-value.ts";
import {
  cancellationCycles,
  cancellationBatch,
  sourceSha256,
  summarizeCancellationDurations,
  type CancellationFrameReceipt,
  type CancellationPublication,
} from "./lsp-resource-cancellation.ts";
import type { NativeAdmissionReceipt, ResourceCorsaGate } from "./lsp-resource-corsa-gate.ts";
import { componentSource, documentUri } from "./lsp-resource-workspace.ts";

function isPublication(value: unknown): value is PublishDiagnosticsParams & object {
  return isRecord(value) && Array.isArray(value.diagnostics) && value.diagnostics.every(isRecord);
}

export async function metadataCompletion(
  session: LspSession,
  uri: string,
  source: string,
  type = "string",
): Promise<JsonRecord> {
  const result = await session.request(
    "textDocument/completion",
    {
      textDocument: { uri },
      position: offsetToPosition(source, source.indexOf(":label=", source.indexOf("<template>"))),
      context: { triggerKind: 1 },
    },
    120_000,
  );
  const items: unknown = Array.isArray(result) ? result : at(result, "items");
  const prop: unknown = Array.isArray(items)
    ? items.find(
        (item: unknown) => isRecord(item) && (item.label === "label" || item.label === ":label"),
      )
    : undefined;
  assert.ok(isRecord(prop), `imported label prop completion missing for ${uri}`);
  assert.match(asString(prop.detail, "detail"), new RegExp(`prop: ${type} \\(required\\)`));
  return prop;
}

export async function bodyEditChurn(
  session: LspSession,
  workspace: string,
  count = 32,
): Promise<{ edits: number; unchanged_prop_completion_replies: number; final_version: number }> {
  const childUri = documentUri(workspace, 0);
  const parentUri = documentUri(workspace, 1);
  let version = 1;
  for (let edit = 0; edit < count; edit++) {
    const value = edit + 1 === count ? 1 : edit + 2;
    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: ++version },
      contentChanges: [
        { text: componentSource(0).replace("checked: number = 1", `checked: number = ${value}`) },
      ],
    });
    await metadataCompletion(session, parentUri, componentSource(1));
  }
  return { edits: count, unchanged_prop_completion_replies: count, final_version: version };
}

type CancellationAttempt = {
  raw_request_id: number;
  request_started_ns: string;
  request_write_returned_ns: string;
};

type RawResponse = { message: JsonRpcMessage; completed_ns: string };

type CancellationSample = CancellationFrameReceipt & {
  cycle: number;
  admission: NativeAdmissionReceipt;
  gate_release: Awaited<ReturnType<ResourceCorsaGate["waitReleased"]>>;
  outer_method: "textDocument/hover";
  outer_request_started_ns: string;
  outer_request_write_returned_ns: string;
  batch_write_started_ns: string;
  batch_write_returned_ns: string;
  raw_response_completed_ns: string;
  raw_outer_response: JsonRpcMessage;
  raw_outer_response_sha256: string;
  raw_response_observer: string;
  elapsed_scope: string;
  elapsed_ms: number;
  post_write_return_ms: number;
};

const peerHoverParams = (workspace: string) => {
  const uri = documentUri(workspace, 1);
  const source = componentSource(1);
  return {
    textDocument: { uri },
    position: offsetToPosition(source, source.indexOf("\nPeer\n") + 2),
  };
};

/** Retain every child diagnostics payload observed during the measured window. */
export function recordCancellationPublications(session: LspSession, workspace: string) {
  const uri = documentUri(workspace, 0);
  const rows: CancellationPublication[] = [];
  let stopped = false;
  const observer = (method: string, value: unknown): void => {
    if (method !== "textDocument/publishDiagnostics" || !isPublication(value)) return;
    if (value.uri !== uri) return;
    const diagnostics = JSON.parse(JSON.stringify(value.diagnostics)) as unknown[];
    rows.push({
      received_order: rows.length + 1,
      received_ns: process.hrtime.bigint().toString(),
      uri,
      version: Number.isSafeInteger(value.version) ? Number(value.version) : null,
      diagnostics,
      diagnostics_sha256: sourceSha256(JSON.stringify(diagnostics)),
    });
  };
  session.notificationObservers.push(observer);
  return {
    count: () => rows.length,
    since: (index: number): CancellationPublication[] => rows.slice(index),
    stop() {
      if (stopped) return;
      stopped = true;
      const index = session.notificationObservers.indexOf(observer);
      if (index >= 0) session.notificationObservers.splice(index, 1);
    },
  };
}

/**
 * Admit one real hover into Corsa, hold its already-produced native response,
 * and only then send the outer cancellation plus one complete edit. Releasing
 * each token before arming the next gives every sample its own proven overlap.
 */
export async function cancelledConcurrentEdits(
  session: LspSession,
  workspace: string,
  initialVersion: number,
  gate: ResourceCorsaGate,
) {
  assert.ok(Number.isSafeInteger(initialVersion) && initialVersion > 0);
  const uri = documentUri(workspace, 0);
  const cleanSource = componentSource(0);
  const brokenSource = cleanSource.replace(
    "const checked: number = 1",
    'const checked: number = "cancelled-edit"',
  );
  assert.notEqual(brokenSource, cleanSource);
  const samples: CancellationSample[] = [];
  let version = initialVersion;
  const waiters = new Map<
    number,
    {
      resolve: (response: RawResponse) => void;
      reject: (error: Error) => void;
      timeout?: NodeJS.Timeout;
    }
  >();
  const responseObserver = (message: JsonRpcMessage): void => {
    if (typeof message.id !== "number") return;
    const waiter = waiters.get(message.id);
    if (!waiter) return;
    if (waiter.timeout) clearTimeout(waiter.timeout);
    waiters.delete(message.id);
    waiter.resolve({ message, completed_ns: process.hrtime.bigint().toString() });
  };
  session.responseObservers.push(responseObserver);
  try {
    for (let cycle = 1; cycle <= cancellationCycles; cycle++) {
      gate.arm(cycle);
      let admission: NativeAdmissionReceipt | undefined;
      let release: Awaited<ReturnType<ResourceCorsaGate["waitReleased"]>> | undefined;
      let attempt: CancellationAttempt | undefined;
      let rawResponse: Promise<RawResponse> | undefined;
      let sample: Omit<CancellationSample, "gate_release"> | undefined;
      let gateReleaseError: unknown;
      try {
        const requestStartedNs = process.hrtime.bigint().toString();
        let rawRequestId = -1;
        const response = session.request(
          "textDocument/hover",
          peerHoverParams(workspace),
          180_000,
          (requestId) => {
            rawRequestId = requestId;
            rawResponse = new Promise<RawResponse>((resolve, reject) => {
              waiters.set(requestId, { resolve, reject });
            });
            void rawResponse.catch(() => undefined);
            return [];
          },
        );
        const settled = response.then(
          (value) => ({ status: "fulfilled" as const, value }),
          (error: unknown) => ({ status: "rejected" as const, error }),
        );
        attempt = {
          raw_request_id: rawRequestId,
          request_started_ns: requestStartedNs,
          request_write_returned_ns: process.hrtime.bigint().toString(),
        };
        assert.ok(rawRequestId > 0 && rawResponse, "outer hover metadata must precede its write");
        admission = await gate.waitEntered(cycle);
        assert.equal(admission.request.method, "textDocument/hover");
        assert.equal(admission.response.id, admission.request.id);

        const source = cycle % 2 === 0 ? cleanSource : brokenSource;
        const batch = cancellationBatch(rawRequestId, uri, ++version, source);
        const rawWaiter = waiters.get(rawRequestId);
        assert.ok(rawWaiter, "raw response observer must remain armed before cancellation");
        rawWaiter.timeout = setTimeout(() => {
          waiters.delete(rawRequestId);
          rawWaiter.reject(new Error(`timed out waiting for raw response ${rawRequestId}`));
        }, 30_000);
        const batchWriteStarted = process.hrtime.bigint();
        session.sendBatch(...batch.messages);
        const batchWriteReturned = process.hrtime.bigint();
        const observed = await rawResponse;
        const observedNs = BigInt(observed.completed_ns);
        assert.ok(batchWriteStarted <= batchWriteReturned);
        assert.ok(batchWriteReturned <= observedNs);
        assert.deepEqual(observed.message, {
          jsonrpc: "2.0",
          id: rawRequestId,
          error: { code: -32800, message: "Canceled" },
        });
        const settlement = await settled;
        assert.equal(settlement.status, "rejected", `outer hover ${rawRequestId} must cancel`);
        if (settlement.status !== "rejected") throw new Error("unreachable fulfilled cancellation");
        assert.ok(settlement.error instanceof LspRequestError);
        assert.equal(settlement.error.id, rawRequestId);
        assert.equal(settlement.error.method, "textDocument/hover");
        assert.equal(settlement.error.code, -32800);
        sample = {
          cycle,
          ...batch.receipt,
          admission,
          outer_method: "textDocument/hover",
          outer_request_started_ns: attempt.request_started_ns,
          outer_request_write_returned_ns: attempt.request_write_returned_ns,
          batch_write_started_ns: batchWriteStarted.toString(),
          batch_write_returned_ns: batchWriteReturned.toString(),
          raw_response_completed_ns: observed.completed_ns,
          raw_outer_response: observed.message,
          raw_outer_response_sha256: sourceSha256(JSON.stringify(observed.message)),
          raw_response_observer:
            "LspSession.responseObservers captured this parsed outer response before promise settlement",
          elapsed_scope:
            "immediately before the one-write cancellation/edit batch until raw outer response observation",
          elapsed_ms: Number(observedNs - batchWriteStarted) / 1e6,
          post_write_return_ms: Number(observedNs - batchWriteReturned) / 1e6,
        };
      } finally {
        try {
          gate.release(cycle);
          if (admission) release = await gate.waitReleased(cycle);
        } catch (releaseError) {
          gateReleaseError = releaseError;
        }
      }
      if (gateReleaseError !== undefined) throw gateReleaseError;
      assert.ok(sample && release, `cycle ${cycle} must retain sample and gate release receipts`);
      samples.push({ ...sample, gate_release: release });
    }
  } finally {
    const observerIndex = session.responseObservers.indexOf(responseObserver);
    if (observerIndex >= 0) session.responseObservers.splice(observerIndex, 1);
    for (const waiter of waiters.values()) {
      if (waiter.timeout) clearTimeout(waiter.timeout);
      waiter.reject(new Error("cancellation response observer stopped"));
    }
    waiters.clear();
  }

  assert.equal(version, initialVersion + cancellationCycles);
  assert.equal(samples.at(-1)?.source_sha256, sourceSha256(cleanSource));
  assert.equal(new Set(samples.map((sample) => sample.raw_request_id)).size, samples.length);
  assert.equal(new Set(samples.map((sample) => sample.admission.native_process.pid)).size, 1);
  for (let index = 1; index < samples.length; index++) {
    assert.ok(
      samples[index].raw_request_id > samples[index - 1].raw_request_id,
      "raw outer request IDs must increase in actual write order",
    );
    assert.deepEqual(
      samples[index].admission.native_process,
      samples[0].admission.native_process,
      "all admitted hovers must use the same actual Corsa process identity",
    );
    assert.deepEqual(
      samples[index].admission.proxy_process,
      samples[0].admission.proxy_process,
      "all admitted hovers must use the same gate proxy process identity",
    );
  }

  return {
    transport:
      "each outer hover is admitted through an actual held Corsa response before $/cancelRequest and full-document didChange share one LspSession stdin.write",
    initial_version: initialVersion,
    final_version: version,
    final_source_sha256: sourceSha256(cleanSource),
    alternating_source_sha256: {
      broken: sourceSha256(brokenSource),
      clean: sourceSha256(cleanSource),
    },
    samples,
    statistics: summarizeCancellationDurations(samples.map((sample) => sample.elapsed_ms)),
  };
}

/** Prove the last clean revision and normal editor requests survive the burst. */
export async function verifyCancellationRecovery(
  session: LspSession,
  workspace: string,
  finalVersion: number,
) {
  const childUri = documentUri(workspace, 0);
  const parentUri = documentUri(workspace, 1);
  const parent = componentSource(1);
  const publish = await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (value) =>
      isPublication(value) &&
      value.uri === childUri &&
      value.version === finalVersion &&
      value.diagnostics.length === 0,
    180_000,
  );
  assert.ok(isPublication(publish));

  const completion = await metadataCompletion(session, parentUri, parent);
  const hover = await session.request("textDocument/hover", peerHoverParams(workspace), 120_000);
  const hoverResult = isRecord(hover) ? hover : null;
  const hoverText = hoverToText(hoverResult);
  assert.match(hoverText, /save: \[value: string\]/);
  return {
    final_version: finalVersion,
    final_diagnostics: publish.diagnostics,
    completion: {
      label: asString(completion.label, "completion label"),
      detail: asString(completion.detail, "completion detail"),
      payload_sha256: sourceSha256(JSON.stringify(completion)),
    },
    hover: { text: hoverText, payload_sha256: sourceSha256(JSON.stringify(hover)) },
  };
}

export async function controlledEdits(
  session: LspSession,
  workspace: string,
  initialVersion = 1,
): Promise<JsonRecord[]> {
  const childUri = documentUri(workspace, 0);
  const parentUri = documentUri(workspace, 1);
  const parent = componentSource(1);
  let child = componentSource(0);
  let version = initialVersion;
  let publications = 0;
  let afterEdit = 0;
  const publicationSequence = new WeakMap<object, number>();
  const sequenceOf = (value: object): number => publicationSequence.get(value) ?? 0;
  session.notificationObservers.push((method, params) => {
    if (method === "textDocument/publishDiagnostics" && isPublication(params)) {
      publicationSequence.set(params, ++publications);
    }
  });
  const proof: JsonRecord[] = [];
  const change = (next: string): void => {
    child = next;
    afterEdit = publications;
    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: ++version },
      contentChanges: [{ text: child }],
    });
  };
  const diagnostic = async (
    uri: string,
    documentVersion: number,
    code: number,
    pattern: RegExp,
  ): Promise<PublishDiagnosticsParams["diagnostics"][number] | undefined> => {
    const params = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (value) =>
        isPublication(value) &&
        sequenceOf(value) > afterEdit &&
        value.uri === uri &&
        value.version === documentVersion &&
        value.diagnostics.some(
          (entry) => entry.code === code && pattern.test(String(entry.message)),
        ),
      180_000,
    );
    assert.ok(isPublication(params));
    return params.diagnostics.find(
      (entry) => entry.code === code && pattern.test(String(entry.message)),
    );
  };
  const start = performance.now();
  change(child.replace("checked: number = 1", 'checked: number = "resource-sentinel"'));
  proof.push({
    edit: "body type error",
    version,
    diagnostic: await diagnostic(
      childUri,
      version,
      2322,
      /Type 'string' is not assignable to type 'number'/,
    ),
  });
  await metadataCompletion(session, parentUri, parent);
  proof.push({ edit: "body change", prop_type: "string", elapsed_ms: performance.now() - start });

  change(child.replace('checked: number = "resource-sentinel"', "checked: number = 1"));
  await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (value) =>
      isPublication(value) &&
      sequenceOf(value) > afterEdit &&
      value.uri === childUri &&
      value.version === version &&
      !value.diagnostics.some((entry) => entry.code === 2322),
    180_000,
  );
  change(child.replace("label: string", "label: number"));
  await metadataCompletion(session, parentUri, parent, "number");
  proof.push({
    edit: "public prop",
    version,
    completion_type: "number",
    diagnostic: await diagnostic(parentUri, 1, 2322, /string.*number/),
  });

  change(
    child
      .replace("label: number", "label: string")
      .replace("save: [value: string]", "save: [value: number]"),
  );
  await metadataCompletion(session, parentUri, parent);
  const hover = await session.request(
    "textDocument/hover",
    {
      textDocument: { uri: parentUri },
      position: offsetToPosition(parent, parent.indexOf("\nPeer\n") + 2),
    },
    120_000,
  );
  const hoverResult = isRecord(hover) ? hover : null;
  assert.match(hoverToText(hoverResult), /save: \[value: number\]/);
  proof.push({ edit: "public emit", version, hover: hoverToText(hoverResult) });

  change(child.replace("publicState: string = 'ready'", "publicState: number = 1"));
  await metadataCompletion(session, parentUri, parent);
  proof.push({
    edit: "public expose",
    version,
    diagnostic: await diagnostic(parentUri, 1, 2339, /toUpperCase.*number/),
  });

  change(componentSource(0));
  await metadataCompletion(session, parentUri, parent);
  await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (value) =>
      isPublication(value) &&
      sequenceOf(value) > afterEdit &&
      value.uri === parentUri &&
      value.version === 1 &&
      !value.diagnostics.some((entry) => entry.code === 2339 || entry.code === 2322),
    180_000,
  );
  proof.push({ edit: "restore interface", version, parent_diagnostics_cleared: true });
  return proof;
}
