import assert from "node:assert/strict";
import { request } from "node:http";
import { createServer as createNetServer } from "node:net";
import test from "node:test";

import {
  resolveNuxtLintDevtoolsOptions,
  setupNuxtLintDevtools,
  type NuxtLintDevtoolsNuxt,
} from "./inspector-devtools.ts";
import { renderNuxtLintInspectorHtml } from "./inspector-view.ts";

type Hook = (...args: unknown[]) => unknown;

function createNuxt() {
  const hooks = new Map<string, Hook>();
  const refreshes: string[] = [];
  const nuxt: NuxtLintDevtoolsNuxt = {
    hook(name, callback) {
      hooks.set(name, callback);
    },
    callHook(name) {
      refreshes.push(name);
    },
  };
  return { hooks, nuxt, refreshes };
}

async function rawRequest(
  url: URL,
  options: { host?: string; method?: string } = {},
): Promise<{
  body: string;
  headers: Record<string, string | string[] | undefined>;
  status: number;
}> {
  return new Promise((resolve, reject) => {
    const current = request(
      url,
      { headers: options.host ? { host: options.host } : undefined, method: options.method },
      (response) => {
        const chunks: Buffer[] = [];
        response.on("data", (chunk) => chunks.push(Buffer.from(chunk)));
        response.on("end", () => {
          resolve({
            body: Buffer.concat(chunks).toString(),
            headers: response.headers,
            status: response.statusCode ?? 0,
          });
        });
      },
    );
    current.once("error", reject);
    current.end();
  });
}

async function availablePort(): Promise<number> {
  const server = createNetServer();
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen({ host: "127.0.0.1", port: 0 }, resolve);
  });
  const address = server.address();
  assert.ok(address && typeof address === "object");
  await new Promise<void>((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });
  return address.port;
}

void test("lint inspector DevTools options default to lazy and validate ports", () => {
  assert.deepEqual(resolveNuxtLintDevtoolsOptions(undefined), {
    enabled: "lazy",
    port: undefined,
  });
  assert.deepEqual(resolveNuxtLintDevtoolsOptions({ enabled: true, port: 4187 }), {
    enabled: true,
    port: 4187,
  });
  for (const port of [0, 65_536, 1.5]) {
    assert.throws(() => resolveNuxtLintDevtoolsOptions({ port }), RangeError);
  }
});

void test("disabled lint inspector does not register hooks", async () => {
  const { hooks, nuxt } = createNuxt();
  assert.equal(await setupNuxtLintDevtools({ enabled: false }, nuxt, () => ({})), undefined);
  assert.equal(hooks.size, 0);
});

void test("lazy lint inspector launches a hardened UI and forwards API requests", async (t) => {
  const { hooks, nuxt, refreshes } = createNuxt();
  const requests: unknown[] = [];
  const controller = await setupNuxtLintDevtools({ enabled: "lazy" }, nuxt, (input) => {
    requests.push(input);
    return {
      schema: "vize.inspector.lint-plan",
      version: 1,
      root: "/project",
      items: [],
      files: [],
    };
  });
  assert.ok(controller);
  t.after(() => controller.close());

  const tabs: unknown[] = [];
  hooks.get("devtools:customTabs")?.(tabs);
  const lazyTab = tabs[0] as {
    requireAuth: boolean;
    view: { actions: Array<{ handle(): Promise<void> }>; type: string };
  };
  assert.equal(lazyTab.requireAuth, true);
  assert.equal(lazyTab.view.type, "launch");
  await lazyTab.view.actions[0]?.handle();
  assert.deepEqual(refreshes, ["devtools:customTabs:refresh"]);

  const viewer = new URL(controller.url() ?? "");
  const html = await rawRequest(viewer);
  assert.equal(html.status, 200);
  assert.match(html.body, /Project-relative file/u);
  assert.equal(html.headers["cache-control"], "no-store");
  assert.equal(html.headers["cross-origin-resource-policy"], "cross-origin");
  assert.equal(html.headers["x-content-type-options"], "nosniff");
  assert.match(String(html.headers["content-security-policy"]), /default-src 'none'/u);

  const api = new URL("api?file=app.vue&fresh=1", viewer);
  const response = await rawRequest(api);
  assert.equal(response.status, 200);
  assert.deepEqual(JSON.parse(response.body), {
    schema: "vize.inspector.lint-plan",
    version: 1,
    root: "/project",
    items: [],
    files: [],
  });
  assert.deepEqual(requests, [{ files: ["app.vue"], fresh: true }]);

  assert.equal((await rawRequest(api, { method: "POST" })).status, 405);
  assert.equal((await rawRequest(api, { host: "example.com" })).status, 421);
  assert.equal((await rawRequest(new URL("api?file=../secret", viewer))).status, 400);
  const head = await rawRequest(viewer, { method: "HEAD" });
  assert.equal(head.status, 200);
  assert.equal(head.body, "");
  assert.ok(Number(head.headers["content-length"]) > 0);
});

void test("eager lint inspector starts immediately and closes with Nuxt", async () => {
  const { hooks, nuxt } = createNuxt();
  const port = await availablePort();
  const controller = await setupNuxtLintDevtools({ enabled: true, port }, nuxt, () => ({
    ok: true,
  }));
  assert.ok(controller?.url());
  assert.equal(new URL(controller.url() ?? "").port, String(port));

  const tabs: unknown[] = [];
  hooks.get("devtools:customTabs")?.(tabs);
  assert.equal((tabs[0] as { view: { type: string } }).view.type, "iframe");
  await hooks.get("close")?.();
  assert.equal(controller?.url(), undefined);
});

void test("lint inspector close waits for an in-flight lazy start", async () => {
  const { nuxt } = createNuxt();
  const controller = await setupNuxtLintDevtools({ enabled: "lazy" }, nuxt, () => ({}));
  assert.ok(controller);
  const starting = controller.start();
  await controller.close();
  await starting;
  assert.equal(controller.url(), undefined);
  await assert.rejects(controller.start(), /closed/u);
});

void test("lint inspector UI avoids HTML injection sinks", () => {
  const html = renderNuxtLintInspectorHtml("nonce");
  assert.doesNotMatch(html, /innerHTML|outerHTML|insertAdjacentHTML/u);
  assert.match(html, /textContent/u);
});

void test("lint inspector hides provider failures and caps responses", async (t) => {
  const first = createNuxt();
  const failed = await setupNuxtLintDevtools({ enabled: true }, first.nuxt, () => {
    throw new Error("private path");
  });
  assert.ok(failed);
  t.after(() => failed.close());
  const error = await rawRequest(new URL("api", failed.url()));
  assert.equal(error.status, 500);
  assert.equal(error.body, '{"error":"inspector_lint_plan_failed"}');
  assert.doesNotMatch(error.body, /private path/u);

  const second = createNuxt();
  const oversized = await setupNuxtLintDevtools({ enabled: true }, second.nuxt, () => ({
    value: "x".repeat(4 * 1024 * 1024),
  }));
  assert.ok(oversized);
  t.after(() => oversized.close());
  assert.equal((await rawRequest(new URL("api", oversized.url()))).status, 413);
});
