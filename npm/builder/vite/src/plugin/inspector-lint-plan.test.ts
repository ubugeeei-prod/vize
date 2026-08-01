import assert from "node:assert/strict";
import { test } from "node:test";

import {
  handleInspectorLintPlanRequest,
  installInspectorLintPlanMiddleware,
  isInspectorLintPlanRequest,
  parseInspectorLintPlanRequest,
  VIZE_INSPECTOR_LINT_PLAN_ENDPOINT,
} from "./inspector-lint-plan.ts";

class MemoryResponse {
  statusCode = 0;
  readonly headers = new Map<string, number | string>();
  body: string | undefined;

  setHeader(name: string, value: number | string): void {
    this.headers.set(name.toLowerCase(), value);
  }

  end(chunk?: string): void {
    this.body = chunk;
  }
}

const silentLogger = { error() {} };

void test("lint-plan middleware is installed only when a provider exists", () => {
  let installed = 0;
  const devServer = {
    middlewares: {
      use() {
        installed += 1;
      },
    },
  } as Parameters<typeof installInspectorLintPlanMiddleware>[0];
  const state = (inspector?: { lintPlan: () => unknown }) =>
    ({
      clientViteBase: "/",
      logger: silentLogger,
      mergedOptions: { inspector },
    }) as Parameters<typeof installInspectorLintPlanMiddleware>[1];

  installInspectorLintPlanMiddleware(devServer, state());
  assert.equal(installed, 0);
  installInspectorLintPlanMiddleware(devServer, state({ lintPlan: () => ({}) }));
  assert.equal(installed, 1);
});

void test("lint-plan request parser accepts bounded relative files and fresh mode", () => {
  assert.equal(isInspectorLintPlanRequest(VIZE_INSPECTOR_LINT_PLAN_ENDPOINT), true);
  assert.equal(isInspectorLintPlanRequest("/__vize/other"), false);
  assert.deepEqual(
    parseInspectorLintPlanRequest(
      `${VIZE_INSPECTOR_LINT_PLAN_ENDPOINT}?file=pages%2Findex.vue&file=pages%2Findex.vue&fresh=1`,
    ),
    { request: { files: ["pages/index.vue"], fresh: true } },
  );
});

void test("lint-plan request parser honors the configured Vite base", () => {
  const endpoint = `/docs/_nuxt${VIZE_INSPECTOR_LINT_PLAN_ENDPOINT}`;
  assert.equal(isInspectorLintPlanRequest(endpoint, "/docs/_nuxt/"), true);
  assert.equal(
    isInspectorLintPlanRequest(VIZE_INSPECTOR_LINT_PLAN_ENDPOINT, "/docs/_nuxt/"),
    false,
  );
  assert.deepEqual(parseInspectorLintPlanRequest(`${endpoint}?file=app.vue`, "/docs/_nuxt/"), {
    request: { files: ["app.vue"], fresh: false },
  });
});

void test("lint-plan request parser rejects ambiguous and unsafe input", () => {
  assert.deepEqual(
    parseInspectorLintPlanRequest(`${VIZE_INSPECTOR_LINT_PLAN_ENDPOINT}?unknown=1`),
    { statusCode: 400, error: "invalid_query" },
  );
  assert.deepEqual(parseInspectorLintPlanRequest(`${VIZE_INSPECTOR_LINT_PLAN_ENDPOINT}?fresh=0`), {
    statusCode: 400,
    error: "invalid_fresh",
  });
  for (const file of ["../secret", "/absolute.vue", "nested//file.vue", "C:%5Cfile.vue"]) {
    assert.deepEqual(
      parseInspectorLintPlanRequest(`${VIZE_INSPECTOR_LINT_PLAN_ENDPOINT}?file=${file}`),
      { statusCode: 400, error: "invalid_file" },
    );
  }
});

void test("lint-plan request parser enforces URL and file-count limits", () => {
  const files = Array.from({ length: 129 }, (_, index) => `file=f${index}.vue`).join("&");
  assert.deepEqual(parseInspectorLintPlanRequest(`${VIZE_INSPECTOR_LINT_PLAN_ENDPOINT}?${files}`), {
    statusCode: 413,
    error: "too_many_files",
  });
  assert.deepEqual(
    parseInspectorLintPlanRequest(`${VIZE_INSPECTOR_LINT_PLAN_ENDPOINT}?file=${"a".repeat(9000)}`),
    { statusCode: 414, error: "request_uri_too_long" },
  );
});

void test("lint-plan handler emits guarded JSON and forwards a normalized request", async () => {
  const response = new MemoryResponse();
  let received: unknown;
  await handleInspectorLintPlanRequest(
    { method: "GET", url: `${VIZE_INSPECTOR_LINT_PLAN_ENDPOINT}?file=app.vue` },
    response,
    (request) => {
      received = request;
      return { schema: "vize.inspector.lint-plan", version: 1 };
    },
    silentLogger,
  );

  assert.deepEqual(received, { files: ["app.vue"], fresh: false });
  assert.equal(response.statusCode, 200);
  assert.equal(response.headers.get("cache-control"), "no-store");
  assert.equal(response.headers.get("x-content-type-options"), "nosniff");
  assert.deepEqual(JSON.parse(response.body ?? ""), {
    schema: "vize.inspector.lint-plan",
    version: 1,
  });
});

void test("lint-plan handler supports HEAD and rejects other methods", async () => {
  const head = new MemoryResponse();
  await handleInspectorLintPlanRequest(
    { method: "HEAD", url: VIZE_INSPECTOR_LINT_PLAN_ENDPOINT },
    head,
    () => ({ ok: true }),
    silentLogger,
  );
  assert.equal(head.statusCode, 200);
  assert.equal(head.body, undefined);
  assert.ok(Number(head.headers.get("content-length")) > 0);

  const post = new MemoryResponse();
  await handleInspectorLintPlanRequest(
    { method: "POST", url: VIZE_INSPECTOR_LINT_PLAN_ENDPOINT },
    post,
    () => ({ ok: true }),
    silentLogger,
  );
  assert.equal(post.statusCode, 405);
  assert.equal(post.headers.get("allow"), "GET, HEAD");
});

void test("lint-plan handler hides provider failures", async () => {
  const response = new MemoryResponse();
  await handleInspectorLintPlanRequest(
    { method: "GET", url: VIZE_INSPECTOR_LINT_PLAN_ENDPOINT },
    response,
    () => {
      throw new Error("do not expose this path");
    },
    silentLogger,
  );
  assert.equal(response.statusCode, 500);
  assert.deepEqual(JSON.parse(response.body ?? ""), { error: "inspector_lint_plan_failed" });
  assert.doesNotMatch(response.body ?? "", /expose|path/);
});

void test("lint-plan handler bounds provider responses", async () => {
  const response = new MemoryResponse();
  await handleInspectorLintPlanRequest(
    { method: "GET", url: VIZE_INSPECTOR_LINT_PLAN_ENDPOINT },
    response,
    () => ({ value: "x".repeat(4 * 1024 * 1024) }),
    silentLogger,
  );
  assert.equal(response.statusCode, 413);
  assert.deepEqual(JSON.parse(response.body ?? ""), { error: "inspector_response_too_large" });
});
