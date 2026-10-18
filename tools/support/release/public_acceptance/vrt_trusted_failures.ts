import assert from "node:assert/strict";
import type { ConsoleMessage } from "playwright";

export interface BrowserFailureRecord {
  kind: "request" | "response" | "failure" | "finished";
  requestId: string;
  url?: string;
  phase?: string;
  method?: string;
  statusCode?: number;
  blockedReason?: string;
  errorText?: string;
  canceled?: boolean;
  frameId?: string;
  loaderId?: string;
  requestType?: string;
  resourceType?: string;
  requestTimestamp?: number;
  timestamp?: number;
  encodedDataLength?: number;
  corsErrorStatus?: { corsError: string; failedParameter: string };
}
export interface ConsoleFailure {
  phase: string;
  text: string;
  url?: string;
  statusCode?: number;
}
interface ExpectedFailure {
  kind: "response" | "failure";
  phase: string;
  url: string;
  statusCode?: number;
  method?: string;
  requestId?: string;
  errorText?: string;
  canceled?: boolean;
}

interface AuthenticationOwner {
  networkId: string;
  frameId: string;
  resourceType: string;
  url: string;
  method: string;
  status: number;
  body: Buffer;
}

function authenticationTerminal(
  owner: Omit<AuthenticationOwner, "body">,
  observed: BrowserFailureRecord[],
): ExpectedFailure[] {
  const responses = observed.filter(
    (r) => r.requestId === owner.networkId && r.kind === "response",
  );
  const terminals = observed.filter(
    (r) => r.requestId === owner.networkId && (r.kind === "failure" || r.kind === "finished"),
  );
  assert.equal(responses.length, 1, "one original authentication response required");
  assert.equal(responses[0].statusCode, 401);
  assert.equal(terminals.length, 1, "one original authentication terminal required");
  const terminal = terminals[0];
  for (const row of [responses[0], terminal]) {
    assert.equal(row.phase, "actual-401");
    assert.equal(row.url, owner.url);
    assert.equal(row.method, "POST");
    assert.equal(row.frameId, owner.frameId);
    assert.equal(row.requestType, "Fetch");
    assert.ok(typeof row.loaderId === "string" && row.loaderId.length > 0);
    assert.ok(typeof row.requestTimestamp === "number" && Number.isFinite(row.requestTimestamp));
  }
  assert.equal(terminal.loaderId, responses[0].loaderId);
  assert.equal(terminal.requestTimestamp, responses[0].requestTimestamp);
  assert.ok(typeof terminal.timestamp === "number" && Number.isFinite(terminal.timestamp));
  assert.ok(typeof terminal.requestTimestamp === "number");
  assert.ok(terminal.timestamp >= terminal.requestTimestamp);
  assert.equal(terminal.blockedReason, undefined);
  assert.equal(terminal.corsErrorStatus, undefined);
  if (terminal.kind === "finished") {
    assert.equal(terminal.errorText, undefined);
    assert.equal(terminal.canceled, undefined);
    assert.ok(
      typeof terminal.encodedDataLength === "number" &&
        Number.isFinite(terminal.encodedDataLength) &&
        terminal.encodedDataLength >= 0,
    );
    return [];
  }
  assert.equal(terminal.resourceType, "Fetch");
  assert.equal(terminal.errorText, "net::ERR_ABORTED");
  assert.equal(terminal.canceled, true);
  return [
    {
      kind: "failure",
      requestId: owner.networkId,
      phase: "actual-401",
      url: owner.url,
      method: "POST",
      errorText: "net::ERR_ABORTED",
      canceled: true,
    },
  ];
}

/** No query, fragment, userinfo, headers, body or credential-bearing opaque URL is retained. */
export function safeBrowserURL(value: string): string | undefined {
  if (!value) return undefined;
  const url = new URL(value);
  if (url.protocol === "http:" || url.protocol === "https:") return `${url.origin}${url.pathname}`;
  if (url.protocol === "blob:") {
    if (url.search || url.hash) return undefined;
    const inner = new URL(url.pathname);
    if (
      inner.protocol === "https:" &&
      !inner.search &&
      !inner.hash &&
      !inner.username &&
      !inner.password
    )
      return `blob:${inner.origin}${inner.pathname}`;
  }
  return undefined;
}

export function browserRequestMetadata(request: { url: string; method: string }, phase: string) {
  const value = safeBrowserURL(request.url),
    url = value ? new URL(value) : undefined;
  return {
    origin: url?.origin,
    pathname: url?.pathname,
    url: value,
    phase,
    method: request.method,
  };
}

export function consoleFailure(message: ConsoleMessage, phase: string): ConsoleFailure {
  const text = message.text();
  const status = /server responded with a status of (\d{3})\b/.exec(text);
  const target = /^Access to fetch at '([^']+)'/.exec(text);
  return {
    phase,
    text,
    url: safeBrowserURL(target?.[1] ?? message.location().url),
    ...(status ? { statusCode: Number(status[1]) } : {}),
  };
}

/** URLs precede requests/revocations; exact401 ownership precedes original response resume. */
export function expectedBrowserFailures() {
  const records: ExpectedFailure[] = [];
  const terminals: Array<Omit<AuthenticationOwner, "body">> = [];
  return {
    records,
    terminals,
    authentication401(owner: AuthenticationOwner) {
      assert.equal(owner.status, 401);
      assert.ok(Buffer.isBuffer(owner.body));
      assert.equal(owner.body.length, 0);
      assert.equal(owner.method, "POST");
      assert.equal(owner.resourceType, "Fetch");
      assert.ok(typeof owner.networkId === "string" && owner.networkId.length > 0);
      assert.ok(typeof owner.frameId === "string" && owner.frameId.length > 0);
      const url = new URL(owner.url);
      assert.equal(url.protocol, "http:");
      assert.equal(url.hostname, "127.0.0.1");
      assert.equal(url.pathname, "/capture");
      assert.equal(url.search + url.hash + url.username + url.password, "");
      assert.equal(terminals.length, 0, "one controlled authentication transaction required");
      const responses = records.filter(
        (row) =>
          row.kind === "response" &&
          row.phase === "actual-401" &&
          row.url === owner.url &&
          row.method === "POST" &&
          row.statusCode === 401,
      );
      assert.equal(responses.length, 1, "one pre-registered authentication response required");
      assert.equal(responses[0].requestId, undefined);
      responses[0].requestId = owner.networkId;
      const { body: _body, ...metadata } = owner;
      terminals.push(metadata);
    },
    deniedSession(endpoint: string) {
      records.push({ kind: "failure", phase: "denied-loopback", url: `${endpoint}/session` });
    },
    capture(endpoint: string, statusCode: 400 | 401) {
      records.push({
        kind: "response",
        phase: `actual-${statusCode}`,
        url: `${endpoint}/capture`,
        method: "POST",
        statusCode,
      });
    },
    revokedBlobs(urls: string[], phase: "actual-401" | "actual-Vue-unmount") {
      assert.ok(urls.length > 0);
      for (const url of urls) {
        assert.equal(safeBrowserURL(url), url);
        assert.ok(url.startsWith("blob:https://"));
        records.push({ kind: "failure", phase, url, method: "GET" });
      }
    },
  };
}
export type ExpectedBrowserFailures = ReturnType<typeof expectedBrowserFailures>;

/** Every actual HTTP/load error must join an exact registered URL/status/request phase. */
export function assertExpectedBrowserFailures(
  observed: BrowserFailureRecord[],
  expected: ExpectedBrowserFailures,
  consoles: ConsoleFailure[],
) {
  const ownedRecords = expected.records.concat(
    expected.terminals.flatMap((owner) => authenticationTerminal(owner, observed)),
  );
  const failures = observed.filter(
    (record) => record.kind === "failure" || (record.statusCode ?? 0) >= 300,
  );
  const matches = (record: BrowserFailureRecord, owned: ExpectedFailure) =>
    record.kind === owned.kind &&
    record.phase === owned.phase &&
    record.url === owned.url &&
    (owned.method === undefined || record.method === owned.method) &&
    record.statusCode === owned.statusCode &&
    (owned.requestId === undefined ||
      (record.requestId === owned.requestId &&
        record.errorText === owned.errorText &&
        record.canceled === owned.canceled));
  for (const record of failures)
    assert.ok(
      ownedRecords.some((owned) => matches(record, owned)),
      "Unowned browser failure",
    );
  for (const owned of ownedRecords)
    assert.ok(
      failures.some((record) => matches(record, owned)),
      "Expected real failure absent",
    );
  for (const message of consoles) {
    const candidates = failures.filter(
      (record) =>
        record.phase === message.phase && (message.url === undefined || record.url === message.url),
    );
    if (message.statusCode !== undefined) {
      assert.ok(
        message.text.startsWith("Failed to load resource:") &&
          candidates.some((record) => record.statusCode === message.statusCode),
        "Console HTTP error lacks exact actual status/URL custody",
      );
    } else {
      assert.ok(
        /Failed to load resource|network access|ERR_FILE_NOT_FOUND|Permission was denied.*loopback/i.test(
          message.text,
        ) && candidates.some((record) => record.kind === "failure"),
        "Console load error lacks exact actual failure custody",
      );
    }
  }
}

interface RefusalResponse {
  url(): string;
  request(): { method(): string };
  status(): number;
}
interface RefusalPause {
  requestId: string;
  request: { url: string; headers: Record<string, string>; method: string };
  responseStatusCode?: number;
  responseErrorReason?: string;
  networkId?: string;
  frameId?: string;
  resourceType?: string;
}
interface RefusalCDP {
  on(event: "Fetch.requestPaused", listener: (event: RefusalPause) => void): unknown;
  off(event: "Fetch.requestPaused", listener: (event: RefusalPause) => void): unknown;
  send(
    method: "Fetch.enable",
    params: { patterns: Array<{ urlPattern: string; requestStage: "Request" | "Response" }> },
  ): Promise<unknown>;
  send(method: "Fetch.disable"): Promise<unknown>;
  send(method: "Fetch.getResponseBody", params: { requestId: string }): Promise<unknown>;
  send(
    method: "Fetch.continueRequest",
    params: {
      requestId: string;
      postData?: string;
      headers?: Array<{ name: string; value: string }>;
    },
  ): Promise<unknown>;
}

/** Only requests are modified; paused response reads own genuine server bytes unchanged. */
export async function actualRequestRefusal(
  page: {
    waitForResponse(predicate: (response: RefusalResponse) => boolean): Promise<RefusalResponse>;
  },
  cdp: RefusalCDP,
  url: string,
  change: { postData?: string; wrongBearer?: string },
  trigger: () => Promise<unknown>,
  onOriginalResponse?: (owner: AuthenticationOwner) => void,
) {
  const parsed = new URL(url);
  assert.equal(parsed.hostname, "127.0.0.1");
  assert.equal(parsed.protocol, "http:");
  assert.equal(parsed.search + parsed.hash, "");
  const pending = new Set<Promise<unknown>>();
  const errors: unknown[] = [];
  let used = 0;
  let responseCount = 0;
  let responseStatus: number | undefined;
  let body: Buffer | undefined;
  let requestIdentity: RefusalPause | undefined;
  let ownership: AuthenticationOwner | undefined;
  const paused = (event: RefusalPause) => {
    const original = event.request;
    const matched = original.url === url && original.method === "POST";
    if (
      matched &&
      (event.responseStatusCode !== undefined || event.responseErrorReason !== undefined)
    ) {
      const reading = (async () => {
        try {
          assert.equal(++responseCount, 1, "exactly one original response is retained");
          assert.equal(event.responseErrorReason, undefined);
          assert.equal(typeof event.responseStatusCode, "number");
          responseStatus = event.responseStatusCode;
          const packet = await cdp.send("Fetch.getResponseBody", { requestId: event.requestId });
          assert.ok(packet && typeof packet === "object" && !Array.isArray(packet));
          assert.deepEqual(Object.keys(packet).sort(), ["base64Encoded", "body"]);
          assert.ok("body" in packet && "base64Encoded" in packet);
          assert.ok(typeof packet.body === "string" && typeof packet.base64Encoded === "boolean");
          const bytes = Buffer.from(packet.body, packet.base64Encoded ? "base64" : "utf8");
          if (packet.base64Encoded) assert.equal(bytes.toString("base64"), packet.body);
          body = bytes;
          assert.ok(typeof event.networkId === "string" && event.networkId.length > 0);
          assert.ok(typeof event.frameId === "string" && event.frameId.length > 0);
          assert.ok(event.resourceType === "Fetch");
          assert.ok(typeof responseStatus === "number");
          assert.ok(requestIdentity);
          assert.equal(event.networkId, requestIdentity.networkId);
          assert.equal(event.frameId, requestIdentity.frameId);
          assert.equal(event.resourceType, requestIdentity.resourceType);
          ownership = {
            networkId: event.networkId,
            frameId: event.frameId,
            resourceType: event.resourceType,
            url: original.url,
            method: original.method,
            status: responseStatus,
            body: bytes,
          };
          onOriginalResponse?.(ownership);
        } catch (error) {
          errors.push(error);
        }
        // Resume the original response even when custody fails. Never fulfill or rewrite it.
        try {
          await cdp.send("Fetch.continueRequest", { requestId: event.requestId });
        } catch (error) {
          errors.push(error);
        }
      })();
      pending.add(reading);
      void reading.then(() => pending.delete(reading));
      return;
    }
    const overrides: { postData?: string; headers?: Array<{ name: string; value: string }> } = {};
    if (matched && used++ === 0) {
      requestIdentity = event;
      if (change.postData !== undefined)
        overrides.postData = Buffer.from(change.postData).toString("base64");
      if (change.wrongBearer !== undefined) {
        const headers = Object.entries(original.headers).filter(
          ([name]) => name.toLowerCase() !== "authorization",
        );
        headers.push(["Authorization", `Bearer ${change.wrongBearer}`]);
        overrides.headers = headers.map(([name, value]) => ({ name, value }));
      }
    }
    const writing = cdp.send("Fetch.continueRequest", { requestId: event.requestId, ...overrides });
    pending.add(writing);
    void writing.then(
      () => pending.delete(writing),
      (error) => {
        errors.push(error);
        pending.delete(writing);
      },
    );
  };
  let failure: { error: unknown } | undefined;
  let result: RefusalResponse | undefined;
  // The disable/drain finalizer is established before enabling any paused request.
  cdp.on("Fetch.requestPaused", paused);
  try {
    await cdp.send("Fetch.enable", {
      patterns: [
        { urlPattern: url, requestStage: "Request" },
        { urlPattern: url, requestStage: "Response" },
      ],
    });
    [result] = await Promise.all([
      page.waitForResponse((r) => r.url() === url && r.request().method() === "POST"),
      trigger(),
    ]);
    assert.equal(used, 1);
  } catch (error) {
    failure = { error };
  } finally {
    // A trigger can fail while a paused body read is still in flight. Keep Fetch valid until it settles.
    while (pending.size) await Promise.allSettled(pending);
    try {
      await cdp.send("Fetch.disable");
    } catch (error) {
      errors.push(error);
    }
    cdp.off("Fetch.requestPaused", paused);
    await Promise.allSettled(pending);
  }
  if (failure || errors.length)
    throw new AggregateError(
      [...(failure ? [failure.error] : []), ...errors],
      "Actual request refusal or cleanup failed",
    );
  assert.equal(responseCount, 1, "missing original paused response");
  assert.ok(result && body && ownership);
  assert.equal(result.status(), responseStatus, "same original HTTP response status");
  return { response: result, body, ownership };
}
