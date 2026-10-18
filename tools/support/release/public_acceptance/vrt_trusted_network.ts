import assert from "node:assert/strict";
import type { CDPSession, Page } from "playwright";
import { type Evidence } from "./vrt_artifacts.ts";
import { sha256 } from "./vrt_fixtures.ts";
import { actualRequestRefusal } from "./vrt_trusted_failures.ts";
export { actualRequestRefusal } from "./vrt_trusted_failures.ts";
import {
  browserRequestMetadata,
  safeBrowserURL,
  type BrowserFailureRecord,
  type ExpectedBrowserFailures,
} from "./vrt_trusted_failures.ts";
import {
  assertBlobsRevoked,
  currentBlobURLs,
  persistedImagesAndReports,
  type PublicSession,
} from "./vrt_trusted_ui.ts";

export interface SpaceRecord extends BrowserFailureRecord {
  eventPhase?: string;
  frameId?: string;
  loaderId?: string;
  requestType?: string;
  resourceType?: string;
  documentURL?: string;
  requestTimestamp?: number;
  timestamp?: number;
  errorText?: string;
  canceled?: boolean;
  corsErrorStatus?: { corsError: string; failedParameter: string };
  origin?: string;
  pathname?: string;
  initiatorIPAddressSpace?: string;
  initiatorIsSecureContext?: boolean;
  resourceIPAddressSpace?: string;
}

/** Actual CDP metadata only. Never retain headers, POST bytes, queries or credentials. */
export async function publicAddressCustody(cdp: CDPSession, phase: () => string = () => "") {
  const records: SpaceRecord[] = [];
  const urls = new Map<string, ReturnType<typeof browserRequestMetadata>>();
  const pending = new Map<string, SpaceRecord[]>();
  function record(id: string, item: Omit<SpaceRecord, "requestId">) {
    const retained = { ...item, requestId: id, eventPhase: phase() };
    if (urls.has(id)) Object.assign(retained, urls.get(id));
    else pending.set(id, [...(pending.get(id) ?? []), retained]);
    records.push(retained);
  }
  const request = ({
    requestId,
    request,
    frameId,
    loaderId,
    type,
    documentURL,
    timestamp,
  }: {
    requestId: string;
    request: { url: string; method: string };
    frameId?: string;
    loaderId?: string;
    type?: string;
    documentURL?: string;
    timestamp?: number;
  }) => {
    const safe = {
      ...browserRequestMetadata(request, phase()),
      frameId,
      loaderId,
      requestType: type,
      documentURL: documentURL ? safeBrowserURL(documentURL) : undefined,
      requestTimestamp: timestamp,
    };
    urls.set(requestId, safe);
    for (const item of pending.get(requestId) ?? []) Object.assign(item, safe);
    pending.delete(requestId);
  };
  const extra = ({
    requestId,
    clientSecurityState,
  }: {
    requestId: string;
    clientSecurityState?: { initiatorIPAddressSpace: string; initiatorIsSecureContext: boolean };
  }) => {
    if (clientSecurityState)
      record(requestId, {
        kind: "request",
        initiatorIPAddressSpace: clientSecurityState.initiatorIPAddressSpace,
        initiatorIsSecureContext: clientSecurityState.initiatorIsSecureContext,
      });
  };
  const response = ({
    requestId,
    resourceIPAddressSpace,
    statusCode,
  }: {
    requestId: string;
    resourceIPAddressSpace: string;
    statusCode: number;
  }) => record(requestId, { kind: "response", resourceIPAddressSpace, statusCode });
  const failed = ({
    requestId,
    blockedReason,
    errorText,
    canceled,
    type,
    timestamp,
    corsErrorStatus,
  }: {
    requestId: string;
    blockedReason?: string;
    errorText: string;
    canceled?: boolean;
    type: string;
    timestamp: number;
    corsErrorStatus?: { corsError: string; failedParameter: string };
  }) =>
    record(requestId, {
      kind: "failure",
      blockedReason,
      errorText,
      canceled,
      resourceType: type,
      timestamp,
      corsErrorStatus,
    });
  const finished = ({
    requestId,
    timestamp,
    encodedDataLength,
  }: {
    requestId: string;
    timestamp: number;
    encodedDataLength: number;
  }) => record(requestId, { kind: "finished", timestamp, encodedDataLength });
  async function close() {
    cdp.off("Network.requestWillBeSent", request);
    cdp.off("Network.requestWillBeSentExtraInfo", extra);
    cdp.off("Network.responseReceivedExtraInfo", response);
    cdp.off("Network.loadingFailed", failed);
    cdp.off("Network.loadingFinished", finished);
    await cdp.send("Network.disable");
  }
  cdp.on("Network.requestWillBeSent", request);
  cdp.on("Network.requestWillBeSentExtraInfo", extra);
  cdp.on("Network.responseReceivedExtraInfo", response);
  cdp.on("Network.loadingFailed", failed);
  cdp.on("Network.loadingFinished", finished);
  try {
    await cdp.send("Network.enable");
    return { records, close };
  } catch (primary) {
    try {
      await close();
    } catch (release) {
      throw new AggregateError([primary, release], "CDP metadata cleanup refused");
    }
    throw primary;
  }
}

/** The published authorization handler returns an original, strictly empty 401 body. */
export function emptyAuthenticationRefusal(status: number, body: Buffer) {
  assert.equal(status, 401);
  assert.ok(Buffer.isBuffer(body));
  assert.equal(body.length, 0, "Original unauthorized response body must be empty");
  return { status, bodyBytes: body.length, bodySha256: sha256(body) };
}

/** Current granular permission only; unsupported pinned Chrome is an honest campaign refusal. */
export async function loopbackPermission(
  page: Page,
  cdp: CDPSession,
  setting: "granted" | "denied",
) {
  assert.equal(await page.evaluate(() => window.isSecureContext), true);
  const { targetInfo } = await cdp.send("Target.getTargetInfo");
  assert.ok(targetInfo.browserContextId);
  await cdp.send("Browser.setPermission", {
    permission: { name: "loopback-network" },
    setting,
    origin: new URL(page.url()).origin,
    browserContextId: targetInfo.browserContextId,
  });
  const actual = await page.evaluate(
    async () =>
      (await navigator.permissions.query({ name: "loopback-network" as PermissionName })).state,
  );
  assert.equal(actual, setting);
  return actual;
}

const wrong = "A".repeat(43);
/** Real server refusals followed by actual renderer authority invalidation; no response substitution. */
export async function authorityRefusals(input: {
  page: Page;
  cdp: CDPSession;
  session: PublicSession;
  root: string;
  artPath: string;
  e: Evidence;
  setPhase: (phase: string) => void;
  expected: ExpectedBrowserFailures;
}) {
  const { page, cdp, session, root, artPath, e, setPhase, expected } = input;
  const origin = new URL(page.url()).origin;
  const records: unknown[] = [];
  setPhase("server-auth-and-input-refusals");
  const stable = await persistedImagesAndReports(root);
  assert.notEqual(session.token, wrong);
  const cases: {
    name: string;
    route: string;
    headers?: HeadersInit;
    body?: string;
    status: number;
  }[] = [
    { name: "missing-bearer", route: "/session", headers: { Origin: origin }, status: 401 },
    {
      name: "wrong-bearer",
      route: "/session",
      headers: { Origin: origin, Authorization: `Bearer ${wrong}` },
      status: 401,
    },
    {
      name: "foreign-origin",
      route: "/session",
      headers: {
        Origin: "https://foreign.invalid",
        Authorization: `Bearer ${session.token}`,
      },
      status: 403,
    },
    {
      name: "null-origin",
      route: "/session",
      headers: { Origin: "null", Authorization: `Bearer ${session.token}` },
      status: 403,
    },
    { name: "malformed-json", route: "/capture", body: "{", status: 400 },
    {
      name: "unknown-art",
      route: "/capture",
      body: JSON.stringify({ artPath: "/unknown.art.vue", update: false }),
      status: 404,
    },
    {
      name: "nonboolean-update",
      route: "/capture",
      body: JSON.stringify({ artPath, update: 1 }),
      status: 400,
    },
  ];
  for (const control of cases) {
    session.assertRunning();
    const response = await fetch(`${session.endpoint}${control.route}`, {
      redirect: "error",
      signal: AbortSignal.timeout(30000),
      method: control.body === undefined ? "GET" : "POST",
      body: control.body,
      headers: control.headers ?? {
        Origin: origin,
        Authorization: `Bearer ${session.token}`,
        "Content-Type": "application/json",
      },
    });
    const body = Buffer.from(await response.arrayBuffer());
    await e.save(`trusted-public/refusals/${control.name}.bin`, body);
    assert.equal(response.status, control.status);
    session.assertRunning();
    assert.deepEqual(await persistedImagesAndReports(root), stable);
    records.push({
      phase: "server-auth-and-input-refusals",
      name: control.name,
      status: response.status,
      successfulCaptureOrPNGMutation: false,
    });
  }
  const blobs = await currentBlobURLs(page);
  setPhase("actual-400");
  expected.capture(session.endpoint, 400);
  const badCapture = await actualRequestRefusal(
    page,
    cdp,
    `${session.endpoint}/capture`,
    { postData: JSON.stringify({ artPath, update: 1 }) },
    () => page.getByRole("button", { name: "Run VRT", exact: true }).click(),
  );
  const raw400 = badCapture.body;
  await e.save("trusted-public/authority/actual-400.json", raw400);
  assert.equal(badCapture.response.status(), 400);
  const error400 = JSON.parse(raw400.toString()) as { error: string };
  assert.equal(typeof error400.error, "string");
  await page.locator(".vrt-error").getByText(error400.error, { exact: true }).waitFor();
  assert.deepEqual(await persistedImagesAndReports(root), stable);
  setPhase("actual-401");
  expected.capture(session.endpoint, 401);
  expected.revokedBlobs(blobs, "actual-401");
  const deniedCapture = await actualRequestRefusal(
    page,
    cdp,
    `${session.endpoint}/capture`,
    { wrongBearer: wrong },
    () => page.getByRole("button", { name: "Run VRT", exact: true }).click(),
    (owner) => expected.authentication401(owner),
  );
  const raw401 = deniedCapture.body;
  await e.save("trusted-public/authority/actual-401.json", raw401);
  const actual401 = emptyAuthenticationRefusal(deniedCapture.response.status(), raw401);
  await page.getByRole("textbox", { name: "VRT endpoint", exact: true }).waitFor();
  for (const selector of [".vrt-summary", ".vrt-results", ".vrt-error"])
    assert.equal(await page.locator(selector).count(), 0, selector);
  assert.equal(
    await page.getByRole("button", { name: "Download JSON report", exact: true }).count(),
    0,
  );
  await assertBlobsRevoked(page, blobs);
  assert.deepEqual(await persistedImagesAndReports(root), stable);
  records.push({
    phase: "actual400-401-empty-authority",
    oldBlobCount: blobs.length,
    allRevoked: true,
    actual400: error400,
    actual401,
    noStaleResults: true,
  });

  session.assertRunning();
  return records;
}

export function assertRealPublicLoopback(records: SpaceRecord[], origin: string, endpoint: string) {
  assert.ok(records.some((v) => v.origin === origin && v.resourceIPAddressSpace === "Public"));
  assert.ok(
    records.some(
      (v) =>
        v.origin === endpoint &&
        v.initiatorIPAddressSpace === "Public" &&
        v.initiatorIsSecureContext === true,
    ),
  );
  assert.ok(records.some((v) => v.origin === endpoint && v.resourceIPAddressSpace === "Loopback"));
  assert.ok(
    records.every(
      (v) =>
        !v.pathname?.includes("@vite") &&
        !v.pathname?.includes("@fs") &&
        !v.pathname?.endsWith(".art.vue"),
    ),
  );
}
