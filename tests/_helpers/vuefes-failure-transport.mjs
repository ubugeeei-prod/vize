import { performance } from "node:perf_hooks";

const selected = (request) =>
  ["document", "script", "stylesheet"].includes(request.resourceType()) ||
  /\.(?:[cm]?js|css)(?:[?#]|$)|[?&]vue(?:[=&]|$)/i.test(request.url());
const message = (error) => String(error?.stack ?? error);

// These listeners observe the original requests; they never fetch or intercept.
export function recordFailureTransport(page) {
  const records = new Map();
  const pending = new Set();
  const failures = [];
  let live = true;
  const ensure = (request) => {
    let entry = records.get(request);
    if (!entry) {
      entry = {
        id: records.size,
        url: request.url(),
        method: request.method(),
        resourceType: request.resourceType(),
        selected: selected(request),
        observedAt: performance.now(),
        redirectedFrom: null,
        complete: false,
        missing: "no response observed before cutoff",
      };
      records.set(request, entry);
      const previous = request.redirectedFrom();
      if (previous) entry.redirectedFrom = ensure(previous).id;
    }
    return entry;
  };
  const guarded =
    (operation) =>
    (...args) => {
      if (!live) return;
      try {
        operation(...args);
      } catch (error) {
        failures.push(message(error));
      }
    };
  const onRequest = guarded(ensure);
  const onFailed = guarded((request) => {
    const entry = ensure(request);
    entry.complete = false;
    entry.requestFailure = request.failure()?.errorText ?? "reason unavailable";
    entry.missing = `request failed: ${entry.requestFailure}`;
    entry.failedAt = performance.now();
  });
  const onResponse = guarded((response) => {
    const entry = ensure(response.request());
    entry.selected ||= /(?:javascript|ecmascript|css|html)/i.test(
      response.headers()["content-type"] ?? "",
    );
    if (!entry.selected) return;
    entry.complete = false;
    if (entry.failedAt === undefined)
      entry.missing = "response observed; headers capture pending at cutoff";
    entry.response = {
      status: response.status(),
      statusText: response.statusText(),
      url: response.url(),
      observedAt: performance.now(),
      fromServiceWorker: response.fromServiceWorker(),
      diskOrMemoryCache: { available: false, reason: "no CDP cache authority collected" },
      headers: null,
      headersArray: null,
      bodyEncoding: "browser-decoded response bytes, not compressed wire bytes",
    };
    const operation = (async () => {
      const [headers, headersArray] = await Promise.all([
        response.allHeaders(),
        response.headersArray(),
      ]);
      if (!live) return;
      Object.assign(entry.response, { headers, headersArray });
      if (entry.failedAt === undefined)
        entry.missing = "response headers captured; decoded body pending at cutoff";
      const body = await response.body();
      if (!live) return;
      entry.body = body;
      entry.complete = entry.failedAt === undefined;
      if (entry.complete) entry.missing = null;
      entry.finishedAt = performance.now();
    })().catch((error) => {
      if (live) {
        entry.response.captureFailure = `${entry.response.headers ? "actual response.body() refused" : "response headers capture failed"}: ${message(error)}`;
        if (entry.failedAt === undefined) entry.missing = entry.response.captureFailure;
      }
    });
    pending.add(operation);
    void operation.finally(() => pending.delete(operation)).catch(() => {});
  });
  try {
    page.on("request", onRequest);
    page.on("requestfailed", onFailed);
    page.on("response", onResponse);
  } catch (error) {
    failures.push(`listener installation refused: ${message(error)}`);
  }
  return {
    inventory() {
      return [...records.values()]
        .filter((entry) => entry.selected)
        .map((entry) => ({
          ...entry,
          response: entry.response ? { ...entry.response } : undefined,
          body: entry.body
            ? { bytes: entry.body.length, missing: "decoded bytes not yet persisted" }
            : null,
        }));
    },
    stop() {
      for (const [name, listener] of [
        ["request", onRequest],
        ["requestfailed", onFailed],
        ["response", onResponse],
      ]) {
        try {
          page.off(name, listener);
        } catch (error) {
          failures.push(`listener removal failed: ${message(error)}`);
        }
      }
      return pending.size;
    },
    async finish() {
      await Promise.all(pending);
    },
    freeze() {
      live = false;
      const requests = [...records.values()].filter((entry) => entry.selected);
      return {
        requests,
        failures: [...failures],
        pendingAtCutoff: pending.size,
        complete: failures.length === 0 && requests.every((entry) => entry.complete),
      };
    },
  };
}
