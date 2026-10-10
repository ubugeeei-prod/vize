import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createServer, request } from "node:http";
import type { Page, Request } from "playwright";

export interface RelayReceipt {
  method: string;
  pathname: string;
  status: number;
  contentType: string;
  bytes: number;
  sha256: string;
}

export interface HeldResponse {
  buffered: Promise<{ receipt: RelayReceipt; body: Buffer }>;
  release(): void;
  released: Promise<void>;
}

interface Hold {
  pathname: string | RegExp;
  status?: number;
  contentType?: string;
}

/** Delay real response bytes; the relay never invents a session, capture, or artifact. */
export async function createLifecycleRelay(endpoint: string) {
  const validateUpstream = (address: string) => {
    const url = new URL(address);
    assert.equal(url.protocol, "http:");
    assert.equal(url.hostname, "127.0.0.1");
    assert.equal(url.pathname, "/");
    assert.equal(url.username + url.password + url.search + url.hash, "");
    assert.ok(Number(url.port) > 0 && Number(url.port) <= 65535);
    return url;
  };
  let backend = validateUpstream(endpoint);
  const receipts: RelayReceipt[] = [];
  const holds: Array<{
    match: Hold;
    selected: boolean;
    buffered(value: { receipt: RelayReceipt; body: Buffer }): void;
    ready: Promise<void>;
    release(): void;
    sent(): void;
  }> = [];
  const server = createServer((incoming, outgoing) => {
    const pathname = new URL(incoming.url ?? "/", endpoint).pathname;
    const target = backend;
    const upstream = request(
      {
        hostname: target.hostname,
        port: target.port,
        method: incoming.method,
        path: incoming.url ?? "/",
        headers: { ...incoming.headers, host: target.host },
      },
      (actual) => {
        // Preserve actual service response headers in memory only; artifacts omit them.
        const headers = { ...actual.headers };
        const contentType = String(actual.headers["content-type"] ?? "");
        const status = actual.statusCode ?? 0;
        const hold = holds.find(
          (item) =>
            !item.selected &&
            (typeof item.match.pathname === "string"
              ? pathname === item.match.pathname
              : item.match.pathname.test(pathname)) &&
            (item.match.status === undefined || item.match.status === status) &&
            (item.match.contentType === undefined || item.match.contentType === contentType),
        );
        if (hold) hold.selected = true;
        const chunks: Buffer[] = [];
        actual.on("data", (chunk: Buffer) => chunks.push(chunk));
        actual.once("error", (error) => outgoing.destroy(error));
        actual.once("end", () => {
          const body = Buffer.concat(chunks);
          const receipt: RelayReceipt = {
            method: incoming.method ?? "GET",
            pathname,
            status,
            contentType,
            bytes: body.length,
            sha256: createHash("sha256").update(body).digest("hex"),
          };
          receipts.push(receipt);
          const send = () => {
            if (!outgoing.destroyed) outgoing.writeHead(status, headers).end(body);
            hold?.sent();
          };
          if (hold) {
            hold.buffered({ receipt, body });
            void hold.ready.then(send);
          } else send();
        });
      },
    );
    upstream.once("error", (error) => outgoing.destroy(error));
    incoming.once("error", (error) => upstream.destroy(error));
    incoming.pipe(upstream);
  });
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  const relayEndpoint = `http://127.0.0.1:${address.port}`;
  return {
    endpoint: relayEndpoint,
    receipts,
    /** Existing in-flight responses retain their original actual upstream and buffered bytes. */
    selectUpstream(address: string) {
      const target = validateUpstream(address);
      assert.notEqual(target.origin, relayEndpoint);
      backend = target;
    },
    holdNext(match: Hold): HeldResponse {
      let buffered!: (value: { receipt: RelayReceipt; body: Buffer }) => void;
      let release!: () => void;
      let sent!: () => void;
      const result = new Promise<{ receipt: RelayReceipt; body: Buffer }>((resolve) => {
        buffered = resolve;
      });
      const ready = new Promise<void>((resolve) => {
        release = resolve;
      });
      const released = new Promise<void>((resolve) => {
        sent = resolve;
      });
      holds.push({ match, selected: false, buffered, ready, release, sent });
      return { buffered: result, release, released };
    },
    async close() {
      for (const hold of holds) hold.release();
      await new Promise<void>((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()));
        server.closeAllConnections();
      });
    },
  };
}

interface ObjectUrlState {
  live: Set<string>;
  created: string[];
  revoked: string[];
  createdAfterUnmount: string[];
  unmounted: boolean;
  restore(): void;
}

/** Observe original native operations with unchanged arguments, bytes, and return values. */
export async function observeObjectUrls(page: Page) {
  await page.addInitScript(() => {
    const create = Reflect.get(URL, "createObjectURL") as typeof URL.createObjectURL;
    const revoke = Reflect.get(URL, "revokeObjectURL") as typeof URL.revokeObjectURL;
    const state: ObjectUrlState = {
      live: new Set(),
      created: [],
      revoked: [],
      createdAfterUnmount: [],
      unmounted: false,
      restore() {
        URL.createObjectURL = create;
        URL.revokeObjectURL = revoke;
      },
    };
    URL.createObjectURL = function (object) {
      const url = Reflect.apply(create, this, [object]) as string;
      state.live.add(url);
      state.created.push(url);
      if (state.unmounted) state.createdAfterUnmount.push(url);
      return url;
    };
    URL.revokeObjectURL = function (url) {
      Reflect.apply(revoke, this, [url]);
      state.live.delete(url);
      state.revoked.push(url);
    };
    Reflect.set(window, "__museaVrtLifecycleUrls", state);
  });
  return {
    async unmounted() {
      await page.evaluate(() => {
        (Reflect.get(window, "__museaVrtLifecycleUrls") as ObjectUrlState).unmounted = true;
      });
    },
    counts() {
      return page.evaluate(() => {
        const state = Reflect.get(window, "__museaVrtLifecycleUrls") as ObjectUrlState;
        return {
          live: state.live.size,
          created: state.created.length,
          revoked: state.revoked.length,
          createdAfterUnmount: state.createdAfterUnmount.length,
        };
      });
    },
    async restore() {
      await page.evaluate(() => {
        (Reflect.get(window, "__museaVrtLifecycleUrls") as ObjectUrlState).restore();
      });
    },
  };
}

/** Resolve for the actual GET's finish or cancellation, without replacing its fetch. */
export function nextArtifactSettlement(page: Page, url: string): Promise<string> {
  return new Promise((resolve) => {
    const settle = (request: Request, outcome: string) => {
      if (request.method() !== "GET" || request.url() !== url) return;
      page.off("requestfinished", finished);
      page.off("requestfailed", failed);
      resolve(outcome);
    };
    const finished = (request: Request) => settle(request, "finished");
    const failed = (request: Request) => settle(request, "failed");
    page.on("requestfinished", finished);
    page.on("requestfailed", failed);
  });
}

export function renderFrames(page: Page): Promise<void> {
  return page.evaluate(
    () =>
      new Promise<void>((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
      }),
  );
}
