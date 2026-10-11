import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import type { Page, Request, Response } from "playwright";

const bodyLimit = 16 * 1024;
const retainedBodyLimit = 64 * 1024;
const jsonLimit = 256 * 1024;
const summaryLimit = 8192 - Buffer.byteLength("MUSEA_NATIVE_RESPONSE_EVIDENCE \n");
const safeHeaders = ["content-type", "content-length", "etag", "cache-control"] as const;
const bounded = (value: string, limit: number) => Buffer.from(value).subarray(0, limit).toString();
const errorName = (error: unknown) =>
  bounded(error instanceof Error ? error.name : typeof error, 64);

export function failedBodyAdmission(length: string | null, remaining: number) {
  if (length === null) return { refusal: "unknown-content-length" };
  if (!/^(0|[1-9]\d{0,5})$/u.test(length)) return { refusal: "invalid-content-length" };
  const bytes = Number(length);
  if (bytes > bodyLimit) return { refusal: "body-limit" };
  if (bytes > remaining) return { refusal: "retained-body-limit" };
  return { bytes };
}

export async function retainFailedBody(response: Pick<Response, "body">, declaredBytes: number) {
  assert.ok(declaredBytes >= 0 && declaredBytes <= bodyLimit);
  const bodyReadStartedAt = Date.now();
  // Response.body buffers the whole exact response. These are retained-byte
  // limits, not a streaming API or a peak-allocation guarantee.
  const body = await response.body();
  const bodyReadFinishedAt = Date.now();
  assert.equal(body.length, declaredBytes, "failed response content-length must match its body");
  return {
    bodyReadStartedAt,
    bodyReadFinishedAt,
    bytes: body.length,
    sha256: createHash("sha256").update(body).digest("hex"),
    encoding: "base64",
    data: body.toString("base64"),
  };
}

interface Failure {
  sequence: number;
  url: string;
  status: number;
  statusText: string;
  responseReceivedAt: number;
  timingAtResponse: ReturnType<Request["timing"]>;
  headers?: Partial<Record<(typeof safeHeaders)[number], string | null>>;
  headersReadStartedAt?: number;
  headersReadFinishedAt?: number;
  requestFinishedAt?: number;
  timingAtFinished?: ReturnType<Request["timing"]>;
  bodyReadStartedAt?: number;
  bodyReadFinishedAt?: number;
  body?: Awaited<ReturnType<typeof retainFailedBody>>;
  refusal?: string;
}

interface CollectionError {
  sequence: number | null;
  stage: string;
  name: string;
}

export function collectNativeEvent(
  errors: CollectionError[],
  sequence: number | null,
  stage: string,
  action: () => void,
) {
  try {
    action();
  } catch (error) {
    errors.push({ sequence, stage, name: errorName(error) });
  }
}

type SummaryFailure = Pick<
  Failure,
  | "url"
  | "status"
  | "statusText"
  | "refusal"
  | "responseReceivedAt"
  | "requestFinishedAt"
  | "bodyReadStartedAt"
  | "bodyReadFinishedAt"
  | "body"
>;

export function nativeFailureSummary(
  failures: readonly SummaryFailure[],
  collectionErrors: readonly CollectionError[],
  omittedFailures: number,
) {
  const rows = failures.slice(0, 8).map((failure) => ({
    url: bounded(failure.url, 256),
    status: failure.status,
    statusText: bounded(failure.statusText, 96),
    refusal: failure.refusal ?? null,
    responseReceivedAt: failure.responseReceivedAt,
    requestFinishedAt: failure.requestFinishedAt,
    bodyReadStartedAt: failure.bodyReadStartedAt,
    bodyReadFinishedAt: failure.bodyReadFinishedAt,
    bodyBytes: failure.body?.bytes ?? null,
    bodySHA256: failure.body?.sha256 ?? null,
    bodyPrefixBase64: failure.body
      ? Buffer.from(failure.body.data, "base64").subarray(0, 288).toString("base64")
      : null,
  }));
  const shown: (typeof rows)[number][] = [];
  const shownErrors: CollectionError[] = [];
  const serialize = () =>
    JSON.stringify({
      failures: shown,
      omitted: failures.length - shown.length + omittedFailures,
      collectionErrors: shownErrors,
      omittedCollectionErrors: collectionErrors.length - shownErrors.length,
    });
  for (const row of rows) {
    shown.push(row);
    if (Buffer.byteLength(serialize()) > summaryLimit) {
      shown.pop();
      break;
    }
  }
  for (const error of collectionErrors) {
    shownErrors.push(error);
    if (Buffer.byteLength(serialize()) > summaryLimit) {
      shownErrors.pop();
      break;
    }
  }
  const summary = serialize();
  assert.ok(Buffer.byteLength(summary) <= summaryLimit, "native failure stdout exceeds its cap");
  return summary;
}

export function observeNativeFailedResponses(page: Page, inventory: Promise<unknown>) {
  const failures: Failure[] = [];
  const collectionErrors: CollectionError[] = [];
  const pending: Promise<void>[] = [];
  const owners = new WeakMap<
    Request,
    { record: Failure; response: Response; headers: Promise<void> }
  >();
  let retainedBytesReserved = 0;
  let omittedFailures = 0;
  let startupInventory: unknown;
  let closed = false;
  const track = (task: Promise<void>, sequence: number | null, stage: string) => {
    pending.push(
      task.catch((error: unknown) => {
        collectionErrors.push({ sequence, stage, name: errorName(error) });
      }),
    );
  };
  track(
    inventory.then((value: unknown) => {
      startupInventory = value;
    }),
    null,
    "startup-inventory",
  );
  const responseReceived = (response: Response) => {
    collectNativeEvent(collectionErrors, failures.length, "response-event", () => {
      if (response.status() < 400) return;
      if (failures.length === 16) {
        omittedFailures++;
        return;
      }
      const url = new URL(response.url());
      const record: Failure = {
        sequence: failures.length,
        url: bounded(url.origin + url.pathname, 1024),
        status: response.status(),
        statusText: bounded(response.statusText(), 128),
        responseReceivedAt: Date.now(),
        timingAtResponse: response.request().timing(),
      };
      failures.push(record);
      const headers = (async () => {
        record.headersReadStartedAt = Date.now();
        const values = await Promise.all(safeHeaders.map((name) => response.headerValue(name)));
        record.headers = Object.fromEntries(
          safeHeaders.map((name, index) => {
            const value = values[index];
            assert.ok(value !== undefined);
            return [name, value === null ? null : bounded(value, 256)];
          }),
        );
        record.headersReadFinishedAt = Date.now();
      })();
      track(headers, record.sequence, "safe-response-headers");
      owners.set(response.request(), { record, response, headers });
    });
  };
  const requestFinished = (request: Request) => {
    const owner = owners.get(request);
    if (!owner) return;
    collectNativeEvent(collectionErrors, owner.record.sequence, "request-finished-event", () => {
      owners.delete(request);
      owner.record.requestFinishedAt = Date.now();
      owner.record.timingAtFinished = request.timing();
      track(
        (async () => {
          await owner.headers;
          const admission = failedBodyAdmission(
            owner.record.headers?.["content-length"] ?? null,
            retainedBodyLimit - retainedBytesReserved,
          );
          if (admission.refusal) {
            owner.record.refusal = admission.refusal;
            return;
          }
          assert.ok(typeof admission.bytes === "number");
          retainedBytesReserved += admission.bytes;
          owner.record.bodyReadStartedAt = Date.now();
          try {
            owner.record.body = await retainFailedBody(owner.response, admission.bytes);
          } finally {
            owner.record.bodyReadFinishedAt = Date.now();
          }
        })(),
        owner.record.sequence,
        "same-finished-response-body",
      );
    });
  };
  page.on("response", responseReceived);
  page.on("requestfinished", requestFinished);
  return {
    async finish() {
      assert.equal(closed, false);
      closed = true;
      page.off("response", responseReceived);
      page.off("requestfinished", requestFinished);
      await Promise.all(pending);
      for (const failure of failures) {
        if (!failure.requestFinishedAt) failure.refusal = "request-not-finished";
      }
      const result = {
        startupInventory,
        failures,
        collectionErrors,
        omittedFailures,
        retainedBytesReserved,
        bodyLimit,
        retainedBodyLimit,
        noQueryRequestHeadersCookiesOrAuthorizationRetained: true,
        noStreamingOrPeakMemoryGuarantee: true,
      };
      const json = JSON.stringify(result, null, 2);
      assert.ok(
        Buffer.byteLength(json) <= jsonLimit,
        "native failure evidence exceeds its retained JSON cap",
      );
      const summary = nativeFailureSummary(failures, collectionErrors, omittedFailures);
      return { json, summary, collectionErrors };
    },
  };
}

export async function nativeStartupInventory(
  config: {
    cacheDir: string;
    optimizeDeps: { include?: string[]; exclude?: string[]; entries?: string | string[] };
  },
  repository: string,
) {
  const startedAt = Date.now();
  const sources = [];
  for (const relative of [
    "npm/builder/vite-musea/gallery/components/MarkdownContent.ts",
    "npm/builder/vite-musea/gallery/components/HighlightedCode.vue",
  ]) {
    const source = await readFile(path.join(repository, relative));
    assert.ok(source.length <= 32 * 1024);
    sources.push({
      path: relative,
      bytes: source.length,
      sha256: createHash("sha256").update(source).digest("hex"),
      knownHighlightImports: [
        ...source.toString().matchAll(/["'](highlight\.js\/[^"']+)["']/gu),
      ].map((match) => match[1]),
      readAt: Date.now(),
    });
  }
  const metadataPath = path.join(config.cacheDir, "deps/_metadata.json");
  let metadata: unknown;
  try {
    const info = await stat(metadataPath);
    assert.ok(info.size <= 64 * 1024);
    const bytes = await readFile(metadataPath);
    assert.ok(bytes.length <= 64 * 1024);
    metadata = {
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      data: JSON.parse(bytes.toString()),
      readAt: Date.now(),
    };
  } catch (error) {
    if (!(error instanceof Error && "code" in error && error.code === "ENOENT")) throw error;
    metadata = { absent: true, readAt: Date.now() };
  }
  return {
    startedAt,
    finishedAt: Date.now(),
    optimizeDeps: {
      include: config.optimizeDeps.include ?? null,
      exclude: config.optimizeDeps.exclude ?? null,
      entries: config.optimizeDeps.entries ?? null,
    },
    cacheDir: config.cacheDir,
    metadata,
    sources,
    readOnlyNoImportFetchPrewarmOrOptimizerWait: true,
  };
}
