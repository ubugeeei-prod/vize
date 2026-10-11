import assert from "node:assert/strict";
import type { ConsoleMessage } from "playwright";

export interface BrowserFailureRecord {
  kind: "request" | "response" | "failure";
  requestId: string;
  url?: string;
  phase?: string;
  method?: string;
  statusCode?: number;
  blockedReason?: string;
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

/** Exact controlled URLs are registered before the corresponding real request/revocation. */
export function expectedBrowserFailures() {
  const records: ExpectedFailure[] = [];
  return {
    records,
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
  const failures = observed.filter(
    (record) => record.kind === "failure" || (record.statusCode ?? 0) >= 300,
  );
  const matches = (record: BrowserFailureRecord, owned: ExpectedFailure) =>
    record.kind === owned.kind &&
    record.phase === owned.phase &&
    record.url === owned.url &&
    (owned.method === undefined || record.method === owned.method) &&
    record.statusCode === owned.statusCode;
  for (const record of failures)
    assert.ok(
      expected.records.some((owned) => matches(record, owned)),
      "Unowned browser failure",
    );
  for (const owned of expected.records)
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
