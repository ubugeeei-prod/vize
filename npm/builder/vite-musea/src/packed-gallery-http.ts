import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { inspect } from "node:util";
import type { Page, Request, Response } from "playwright";

export interface PackedGalleryHttpRecord {
  url: string;
  status: number;
  headers: Record<string, string>;
  requestHeaders: Record<string, string>;
  bytes?: number;
  sha256?: string;
  cacheValidation?: { ifNoneMatch: string; responseEtag: string | null };
  cachedBody?: {
    url: string;
    status: 200;
    etag: string;
    bytes: number;
    sha256: string;
  };
}

export interface PackedGalleryHttpContext {
  origin: string;
  caseOutput: string;
  responses: PackedGalleryHttpRecord[];
  pending: Promise<void>[];
  httpErrors: string[];
  sha256: (body: Buffer) => string;
  onBodyCapture?: (event: string, response: PackedGalleryResponse, result?: unknown) => void;
}

type PackedGalleryResponse = Pick<Response, "url" | "status" | "allHeaders" | "body"> & {
  request(): Pick<Request, "allHeaders" | "redirectedTo">;
};

/** Finish original response bodies before their document or iframe is replaced. */
export async function drainPackedGalleryHttp(pending: Promise<void>[]): Promise<void> {
  for (const capture of pending) await capture;
}

export function createPackedGalleryHttpObserver(context: PackedGalleryHttpContext) {
  const { origin, caseOutput, responses, pending, httpErrors, sha256, onBodyCapture } = context;
  const owners = new Map<string, Promise<PackedGalleryHttpRecord | undefined>>();
  return (response: PackedGalleryResponse) => {
    if (!response.url().startsWith(origin + "/")) return;
    const status = response.status();
    const priorOwner = status === 304 ? owners.get(response.url()) : undefined;
    const work = (async () => {
      if (status === 200) onBodyCapture?.("body-start", response);
      // Admit the original body before metadata IPC yields to document/iframe navigation.
      // Settle its result immediately, then retain metadata even when that body is unavailable.
      const [captured, headers, requestHeaders] = await Promise.all([
        status === 200
          ? response.body().then(
              (body) => {
                onBodyCapture?.("body-finished", response, { bytes: body.length });
                return { body };
              },
              (error: unknown) => {
                onBodyCapture?.("body-error", response, error);
                return { error };
              },
            )
          : undefined,
        response.allHeaders(),
        response.request().allHeaders(),
      ]);
      const record: PackedGalleryHttpRecord = {
        url: response.url(),
        status,
        headers,
        requestHeaders,
      };
      responses.push(record);
      assert.ok(status === 200 || status === 304, `Unexpected HTTP ${status}: ${response.url()}`);
      if (status === 304) {
        assert.equal(headers.location, undefined);
        assert.equal(response.request().redirectedTo(), null);
        assert.ok(priorOwner, `Unowned cached response: ${response.url()}`);
        const prior = await priorOwner;
        assert.ok(prior && prior.status === 200 && prior.bytes !== undefined && prior.sha256);
        assert.equal(prior.url, response.url());
        assert.ok(prior.headers.etag);
        assert.equal(requestHeaders["if-none-match"], prior.headers.etag);
        if (headers.etag !== undefined) assert.equal(headers.etag, prior.headers.etag);
        record.cacheValidation = {
          ifNoneMatch: requestHeaders["if-none-match"],
          responseEtag: headers.etag ?? null,
        };
        record.cachedBody = {
          url: prior.url,
          status: 200,
          etag: prior.headers.etag,
          bytes: prior.bytes,
          sha256: prior.sha256,
        };
        return;
      }
      assert.ok(captured);
      if ("error" in captured) throw captured.error;
      const body = captured.body;
      const digest = sha256(body);
      await mkdir(path.join(caseOutput, "http-bodies"), { recursive: true });
      await writeFile(path.join(caseOutput, "http-bodies", `${digest}.bin`), body);
      record.bytes = body.length;
      record.sha256 = digest;
      return record;
    })();
    if (status === 200) owners.set(response.url(), work);
    pending.push(
      work
        .then(() => undefined)
        .catch((error) => {
          httpErrors.push(String(error));
        }),
    );
  };
}

/** Passive custody only: no routing, cache changes, body substitution or navigation waits. */
export async function tracePackedGalleryHttp(page: Page, events: unknown[], phase: () => string) {
  const started = performance.now();
  const ids = new WeakMap<object, number>();
  let next = 0;
  function requestId(request: object) {
    if (!ids.has(request)) ids.set(request, ++next);
    return ids.get(request)!;
  }
  function record(event: string, fields: object) {
    events.push({
      sequence: events.length,
      elapsedMs: performance.now() - started,
      phase: phase(),
      pageUrl: page.url(),
      event,
      ...fields,
    });
  }
  function errorFields(error: unknown) {
    return error instanceof Error
      ? {
          name: error.name,
          message: error.message,
          stack: error.stack,
          ...Object.fromEntries(Object.entries(error)),
          cause: inspect(error.cause, { depth: null, customInspect: false }),
          raw: inspect(error, { depth: null, showHidden: true, customInspect: false }),
        }
      : { type: typeof error, raw: inspect(error, { depth: null, customInspect: false }) };
  }
  function requestFields(request: Request) {
    const fields = {
      request: requestId(request),
      url: request.url(),
      method: request.method(),
      resourceType: request.resourceType(),
      navigation: request.isNavigationRequest(),
    };
    try {
      const frame = request.frame();
      return { ...fields, frameUrl: frame.url(), mainFrame: frame === page.mainFrame() };
    } catch (error) {
      // Initial navigation requests can precede their public Frame object.
      return { ...fields, frameError: errorFields(error) };
    }
  }
  page.on("request", (request) => record("request", requestFields(request)));
  page.on("requestfinished", (request) => record("request-finished", requestFields(request)));
  page.on("requestfailed", (request) =>
    record("request-failed", { ...requestFields(request), failure: request.failure() }),
  );
  page.on("framenavigated", (frame) =>
    record("frame-navigated", { url: frame.url(), mainFrame: frame === page.mainFrame() }),
  );
  page.on("load", () => record("page-load", {}));
  const session = await page.context().newCDPSession(page);
  session.on("Page.frameNavigated", ({ frame }) => record("cdp-frame-navigated", { frame }));
  session.on("Page.navigatedWithinDocument", (event) => record("cdp-same-document", event));
  await session.send("Page.enable");
  return {
    close: () => session.detach(),
    capture(event: string, response: PackedGalleryResponse, result?: unknown) {
      record(event, {
        request: requestId(response.request()),
        url: response.url(),
        status: response.status(),
        result: event === "body-error" ? errorFields(result) : result,
      });
    },
  };
}
