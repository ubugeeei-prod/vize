import assert from "node:assert/strict";
import { once } from "node:events";
import { authorizeSession } from "../../../npm/builder/vite-musea/src/cli/service/auth.ts";
import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import { readRawBlob } from "../../../tools/support/release/public_acceptance/plan.ts";
import {
  createVrtFixture,
  materializeVrtInputs,
  sha256,
  vrtSources,
  writeVrtConfig,
} from "../../../tools/support/release/public_acceptance/vrt_fixtures.ts";
import {
  assertVrtCustody,
  cleanup,
  evidence,
} from "../../../tools/support/release/public_acceptance/vrt_artifacts.ts";
import {
  actualRequestRefusal,
  emptyAuthenticationRefusal,
} from "../../../tools/support/release/public_acceptance/vrt_trusted_network.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const head = execFileSync("git", ["--no-replace-objects", "-C", root, "rev-parse", "HEAD"], {
  encoding: "utf8",
}).trim();

/** Inert CDP lifetime model only: no browser, renderer, server or HTTP response is substituted. */
function refusalLifetime(
  options: {
    base64?: boolean;
    bodyError?: Error;
    disableError?: Error;
    packet?: unknown;
    missingResponsePause?: boolean;
    responseStatus?: number;
    status?: 400 | 401;
    triggerError?: Error;
    missingNetworkId?: boolean;
    responseNetworkId?: string;
    responseFrame?: string;
    responseResourceType?: string;
  } = {},
) {
  const url = "http://127.0.0.1:43210/capture";
  const body = Buffer.from('{"error":"original unauthorized — 原本"}\n');
  const calls: string[] = [];
  type Listener = Parameters<Parameters<typeof actualRequestRefusal>[1]["on"]>[1];
  let listener: Listener | undefined;
  let stages: string[] = [];
  let enabled = false;
  let pausedResponse = false;
  const response = {
    url: () => url,
    request: () => ({ method: () => "POST" }),
    status: () => options.status ?? 401,
    async body() {
      assert.ok(
        enabled && pausedResponse,
        "late original body unavailable after response resume or Fetch.disable",
      );
      return body;
    },
  };
  let deliver: (() => void) | undefined;
  const page = {
    waitForResponse(predicate: (value: typeof response) => boolean) {
      return new Promise<typeof response>((resolve) => {
        deliver = () => {
          assert.ok(predicate(response));
          resolve(response);
        };
      });
    },
  };
  const request = { url, method: "POST", headers: { Authorization: "Bearer original" } };
  const cdp: Parameters<typeof actualRequestRefusal>[1] = {
    on(_event, callback) {
      listener = callback;
      return this;
    },
    off(_event, callback) {
      assert.equal(listener, callback);
      listener = undefined;
      calls.push("off");
      return this;
    },
    async send(
      method: string,
      params?: {
        patterns?: Array<{ urlPattern: string; requestStage: string }>;
        requestId?: string;
        postData?: string;
        headers?: Array<{ name: string; value: string }>;
      },
    ) {
      calls.push(method);
      if (method === "Fetch.enable") {
        enabled = true;
        assert.ok(params?.patterns);
        assert.ok(params.patterns.every((pattern) => pattern.urlPattern === url));
        stages = params.patterns.map((p) => p.requestStage);
      } else if (method === "Fetch.getResponseBody") {
        assert.ok(enabled && pausedResponse);
        assert.deepEqual(params, { requestId: "response" });
        if (options.triggerError) await new Promise<void>((resolve) => setImmediate(resolve));
        assert.ok(enabled && pausedResponse, "Fetch remains valid throughout body acquisition");
        if (options.bodyError) throw options.bodyError;
        calls.push("body-owned");
        return Object.hasOwn(options, "packet")
          ? options.packet
          : {
              body: body.toString(options.base64 === false ? "utf8" : "base64"),
              base64Encoded: options.base64 !== false,
            };
      } else if (method === "Fetch.continueRequest") {
        assert.ok(params);
        if (params.requestId === "preflight") {
          assert.deepEqual(params, { requestId: "preflight" });
          if (stages.includes("Response")) {
            assert.ok(listener);
            listener({
              requestId: "preflight-response",
              request: { ...request, method: "OPTIONS" },
              responseStatusCode: 204,
            });
          }
        } else if (params.requestId === "preflight-response") {
          assert.deepEqual(params, { requestId: "preflight-response" });
        } else if (params.requestId === "request") {
          if (options.status === 400)
            assert.equal(params.postData, Buffer.from('{"bad":true}').toString("base64"));
          else assert.deepEqual(params.headers, [{ name: "Authorization", value: "Bearer wrong" }]);
          if (stages.includes("Response") && !options.missingResponsePause) {
            pausedResponse = true;
            assert.ok(listener);
            listener({
              requestId: "response",
              request,
              networkId:
                options.responseNetworkId ?? (options.missingNetworkId ? undefined : "network"),
              frameId: options.responseFrame ?? "frame",
              resourceType: options.responseResourceType ?? "Fetch",
              responseStatusCode: options.responseStatus ?? options.status ?? 401,
            });
          } else {
            assert.ok(deliver);
            deliver();
          }
        } else {
          assert.deepEqual(
            params,
            { requestId: "response" },
            "resume original response without overrides",
          );
          assert.ok(deliver);
          pausedResponse = false;
          deliver();
        }
      } else if (method === "Fetch.disable") {
        enabled = false;
        if (options.disableError) throw options.disableError;
      } else assert.fail(`unexpected protocol operation: ${method}`);
      return {};
    },
  };
  const trigger = async () => {
    assert.ok(listener);
    listener({ requestId: "preflight", request: { ...request, method: "OPTIONS" } });
    listener({
      requestId: "request",
      request,
      networkId: "network",
      frameId: "frame",
      resourceType: "Fetch",
    });
    if (options.triggerError) throw options.triggerError;
  };
  return { page, cdp, trigger, body, calls, response, url, stages: () => stages.slice() };
}

test("the same original refusal body is owned before resuming and disabling interception", async () => {
  for (const base64 of [false, true])
    for (const status of [400, 401] as const) {
      const f = refusalLifetime({ base64, status });
      let responseOwned = false;
      const captured = await actualRequestRefusal(
        f.page,
        f.cdp,
        f.url,
        status === 400 ? { postData: '{"bad":true}' } : { wrongBearer: "wrong" },
        f.trigger,
        (owner) => {
          assert.equal(f.calls.at(-1), "body-owned");
          assert.deepEqual(owner, {
            networkId: "network",
            frameId: "frame",
            resourceType: "Fetch",
            url: f.url,
            method: "POST",
            status,
            body: f.body,
          });
          responseOwned = true;
        },
      );
      assert.equal(responseOwned, true);
      assert.deepEqual(captured.body, f.body);
      assert.deepEqual(f.stages(), ["Request", "Response"]);
      assert.equal(captured.response.status(), status);
      const acquired = f.calls.indexOf("Fetch.getResponseBody"),
        disabled = f.calls.indexOf("Fetch.disable");
      assert.ok(acquired > 0 && acquired < disabled && disabled < f.calls.indexOf("off"));
      assert.ok(acquired < f.calls.lastIndexOf("Fetch.continueRequest"));
      await assert.rejects(f.response.body(), /late original body unavailable/);
    }
});

test("original body read and cleanup failures stay failures and release the paused response", async () => {
  for (const disableError of [undefined, new Error("disable refused")]) {
    const bodyError = new Error("original body unavailable");
    const f = refusalLifetime({ bodyError, disableError });
    await assert.rejects(
      actualRequestRefusal(f.page, f.cdp, f.url, { wrongBearer: "wrong" }, f.trigger),
      (error: unknown) => {
        assert.ok(error instanceof AggregateError);
        assert.ok(error.errors.includes(bodyError));
        if (disableError) assert.ok(error.errors.includes(disableError));
        return true;
      },
    );
    assert.equal(f.calls.filter((x) => x === "Fetch.continueRequest").length, 4);
    assert.ok(f.calls.includes("Fetch.disable") && f.calls.includes("off"));
  }
});

test("trigger failure cannot disable Fetch while its original body is being acquired", async () => {
  const triggerError = new Error("trigger failed after starting original request");
  const f = refusalLifetime({ triggerError });
  await assert.rejects(
    actualRequestRefusal(f.page, f.cdp, f.url, { wrongBearer: "wrong" }, f.trigger),
    (error: unknown) => {
      assert.ok(error instanceof AggregateError);
      assert.deepEqual(error.errors, [triggerError]);
      return true;
    },
  );
  const owned = f.calls.indexOf("body-owned");
  assert.ok(owned >= 0 && owned < f.calls.indexOf("Fetch.disable"));
  assert.ok(f.calls.includes("off"));
});

test("missing original response pause, malformed body and mismatched status cannot qualify", async () => {
  for (const options of [
    { missingResponsePause: true },
    { responseStatus: 200 },
    { missingNetworkId: true },
    { responseNetworkId: "foreign" },
    { responseNetworkId: "" },
    { responseFrame: "foreign" },
    { responseFrame: "" },
    { responseResourceType: "Document" },
    { packet: null },
    { packet: { body: 4, base64Encoded: true } },
    { packet: { body: "same", base64Encoded: "false" } },
    { packet: { body: "not canonical base64!", base64Encoded: true } },
    { packet: { body: "", base64Encoded: false, extra: true } },
  ]) {
    const f = refusalLifetime(options);
    await assert.rejects(
      actualRequestRefusal(f.page, f.cdp, f.url, { wrongBearer: "wrong" }, f.trigger),
    );
    assert.ok(f.calls.includes("Fetch.disable") && f.calls.includes("off"));
  }
});
test("original authorization HTTP 401 is strictly empty, not a JSON record", async (t) => {
  const origin = "https://authored.example";
  const server = createServer((request, response) => {
    const host = request.headers.host;
    assert.ok(host);
    assert.equal(authorizeSession(request, response, origin, host, "original-token"), false);
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  t.after(() => server[Symbol.asyncDispose]());
  const address = server.address();
  assert.ok(address && typeof address === "object");
  const response = await fetch(`http://127.0.0.1:${address.port}/capture`, {
    method: "POST",
    headers: { origin, authorization: "Bearer wrong" },
  });
  const body = Buffer.from(await response.arrayBuffer());
  assert.deepEqual(emptyAuthenticationRefusal(response.status, body), {
    status: 401,
    bodyBytes: 0,
    bodySha256: sha256(Buffer.alloc(0)),
  });
  assert.throws(() => JSON.parse(body.toString()));
  for (const invalid of [Buffer.from("{}"), Buffer.from("\n"), Buffer.from([0])])
    assert.throws(() => emptyAuthenticationRefusal(401, invalid));
  for (const status of [200, 400, 403])
    assert.throws(() => emptyAuthenticationRefusal(status, body));
});

async function consumer(t: TestContext) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "vize-vrt-guard-law-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const inputs = materializeVrtInputs(directory, (file) => readRawBlob(root, head, file));
  return { directory, inputs };
}

// These source/custody laws load no product, compiler, renderer, API or native provider.
test("raw release VRT seeds remain whole and sealed while the runtime workspace changes", async (t) => {
  const { directory, inputs } = await consumer(t);
  assert.deepEqual(
    inputs.map((item) => item.sourceFile),
    vrtSources,
  );
  for (const input of inputs) {
    const bytes = await readFile(path.join(directory, input.file));
    assert.deepEqual(bytes, readRawBlob(root, head, input.sourceFile));
    assert.equal(sha256(bytes), input.sha256);
  }
  const f = createVrtFixture(directory);
  assert.equal(f.settings.threshold, 100);
  assert.deepEqual(f.settings.viewports, [{ name: "authored-compact", width: 320, height: 180 }]);
  assert.equal(f.delay, 300);
  await writeFile(f.arts.left, f.authored.left.toString().replace("#0000ff", "#00ff00"));
  await rm(f.arts.right);
  writeVrtConfig(f, true);
  for (const input of inputs)
    assert.equal(sha256(await readFile(path.join(directory, input.file))), input.sha256);
  assert.ok(f.root.startsWith(directory + path.sep));
  assert.ok(inputs.every((item) => item.file.startsWith("vrt-inputs/")));
});

test("altered authored Art or configured VRT options refuse before runtime imports", async (t) => {
  for (const kind of ["art", "settings"] as const) {
    const { directory } = await consumer(t);
    const target = path.join(
      directory,
      "vrt-inputs",
      kind === "art" ? "left/Button.art.vue" : "gallery-vrt-options.json",
    );
    if (kind === "art")
      await writeFile(target, (await readFile(target, "utf8")).replace("Left", "Changed"));
    else {
      const config = JSON.parse(await readFile(target, "utf8"));
      config.threshold = 99;
      await writeFile(target, JSON.stringify(config));
    }
    assert.throws(() => createVrtFixture(directory), { name: "AssertionError" });
  }
});

test("an evidence overwrite failure still closes the actual HTTP listener and retains later bytes", async (t) => {
  const { directory } = await consumer(t);
  const e = await evidence(path.join(directory, "evidence"));
  await e.save("body.bin", Buffer.from([0, 255, 128, 10]));
  const server = createServer((_request, response) => response.end("original"));
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(() => {
    if (server.listening) server.close();
  });
  await assert.rejects(
    cleanup(e, "negative-cleanup", [
      { name: "overwrite", run: () => e.save("body.bin", "changed") },
      {
        name: "real-http-close",
        run: () =>
          new Promise<void>((resolve, reject) =>
            server.close((error) => (error ? reject(error) : resolve())),
          ),
      },
      { name: "later-evidence", run: () => e.save("later.bin", Buffer.from([13, 10, 0])) },
    ]),
    { name: "AggregateError" },
  );
  assert.equal(server.listening, false);
  assert.deepEqual(await readFile(path.join(e.output, "body.bin")), Buffer.from([0, 255, 128, 10]));
  assert.deepEqual(await readFile(path.join(e.output, "later.bin")), Buffer.from([13, 10, 0]));
  assert.deepEqual(
    e.records
      .filter((item) => (item as { phase: string }).phase === "negative-cleanup")
      .map((item) => (item as { operation: string }).operation),
    ["overwrite"],
  );
  await assert.rejects(e.save("../escape.bin", "invalid"), { name: "AssertionError" });
});

test("VRT sealing refuses incomplete success and any different installed package or provider bytes", () => {
  const native = {
    version: "guard-only",
    packages: [{ name: "guard-only", integrity: "sha512-original" }],
    loaded: { entry: "guard-only.node", sha256: "a".repeat(64) },
  };
  const vrt = { ...native, schema: "vize-public-installed-musea-vrt-v1", success: true };
  assert.doesNotThrow(() => assertVrtCustody(vrt, native));
  for (const altered of [
    {},
    { ...vrt, success: false },
    { ...vrt, version: "other" },
    { ...vrt, schema: "other" },
    { ...vrt, packages: [] },
    { ...vrt, loaded: { ...native.loaded, sha256: "b".repeat(64) } },
  ])
    assert.throws(() => assertVrtCustody(altered, native), { name: "AssertionError" });
});

test("VRT runs independently after completed original probes while the original strict seal rejects failure", async () => {
  const workflow = parse(
    await readFile(path.join(root, ".github/workflows/release-public-acceptance.yml"), "utf8"),
  );
  const job = workflow.jobs["public-consumer"];
  const step = job.steps.find((item: { id: string }) => item.id === "vrt");
  assert.equal(job.steps.filter((item: { id: string }) => item.id === "vrt").length, 1);
  assert.equal(job["timeout-minutes"], 30);
  assert.equal(job["continue-on-error"], undefined);
  assert.ok(job.steps.every((item: Record<string, unknown>) => !item["continue-on-error"]));
  assert.equal(
    job.steps.filter((item: { run?: string }) => item.run?.includes("npm install ")).length,
    1,
  );
  assert.equal(
    step.run,
    [
      'export VRT_RETAINED_OUTPUT="$PUBLIC_OUTPUT"',
      'PUBLIC_OUTPUT="$(mktemp -d "$RUNNER_TEMP/vize-public-vrt.XXXXXX")"',
      "vrt_status=0",
      'node vrt_observer.ts "$PUBLIC_VERSION" "$PUBLIC_OUTPUT/vrt.json" 3>&2 > "$PUBLIC_OUTPUT/vrt.log" 2>&1 || vrt_status=$?',
      "printf 'VRT observer exited: %d\\n' \"${vrt_status:-0}\" >&2",
      'if ! node vrt_retained.ts "$PUBLIC_OUTPUT/vrt.log" "$VRT_RETAINED_OUTPUT"; then',
      '  if [ "$vrt_status" -eq 0 ]; then exit 1; fi',
      '  exit "$vrt_status"',
      "fi",
      'exit "$vrt_status"',
      "",
    ].join("\n"),
  );
  assert.equal(step["working-directory"], "${{ env.PUBLIC_CONSUMER }}");
  const prerequisiteIds = ["guard", "prepare", "install", "installed", "chromium"];
  const ids = [...prerequisiteIds, "native", "browser", "vrt"];
  const evaluate = (
    condition: string,
    overrides: Record<string, string> = {},
    cancelled = false,
    success = true,
  ) => {
    const steps = Object.fromEntries(
      [...ids, "lsp", "formatter"].map((id) => [id, { outcome: overrides[id] ?? "success" }]),
    );
    return runInNewContext(
      condition.slice(3, -2),
      { steps, cancelled: () => cancelled, success: () => success },
      { timeout: 100 },
    );
  };
  for (const native of ["success", "failure"])
    for (const browser of ["success", "failure"])
      assert.equal(evaluate(step.if, { native, browser }, false, false), true);
  for (const id of ids.filter((id) => id !== "vrt"))
    for (const outcome of ["skipped", "cancelled"])
      assert.equal(evaluate(step.if, { [id]: outcome }), false);
  for (const id of prerequisiteIds) assert.equal(evaluate(step.if, { [id]: "failure" }), false);
  assert.equal(evaluate(step.if, {}, true), false);
  const seal = job.steps.find(
    (item: { name: string }) => item.name === "Seal installed public acceptance",
  );
  assert.equal(evaluate(seal.if), true);
  assert.equal(evaluate(seal.if, { vrt: "failure" }, false, false), false);
  for (const id of ["lsp", "formatter"])
    for (const outcome of ["failure", "skipped", "cancelled"])
      assert.equal(evaluate(seal.if, { [id]: outcome }), false);
});
