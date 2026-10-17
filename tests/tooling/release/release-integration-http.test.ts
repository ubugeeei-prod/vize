import assert from "node:assert/strict";
import { test } from "node:test";
import { createIntegrationReadApi } from "../../../tools/support/release/qualify_check/integration-http.ts";

const identity = { repository: "owner/repo", token: "controlled-test-token" };

test("actual GET retry sleep expires at the authenticated original deadline", async () => {
  let time = 1_000;
  let requests = 0;
  const sleeps: number[] = [];
  const transport = createIntegrationReadApi({
    ...identity,
    now: () => time,
    sleep: async (milliseconds) => {
      sleeps.push(milliseconds);
      time += milliseconds;
    },
    fetchImpl: async (_url, init) => {
      assert.equal(init?.method, "GET");
      requests += 1;
      return Response.json({ message: "controlled transient failure" }, { status: 503 });
    },
  });
  transport.setPendingDeadline(1_500);
  assert.throws(() => transport.setPendingDeadline(1_501), /cannot be renewed/);
  assert.throws(() => transport.setPendingDeadline(1_499), /cannot be renewed/);
  await assert.rejects(transport.api.read("actions/runs"), /original Q transport deadline expired/);
  assert.equal(requests, 1);
  assert.deepEqual(sleeps, [500]);
  assert.throws(() => transport.setPendingDeadline(1_500), /expired/);
});

test("an in-flight GET aborts at original expiry and cannot start a retry", async () => {
  let time = 1_000;
  let requests = 0;
  let aborted = false;
  const sleeps: number[] = [];
  const transport = createIntegrationReadApi({
    ...identity,
    now: () => time,
    sleep: async (milliseconds) => {
      sleeps.push(milliseconds);
    },
    fetchImpl: (_url, init) => {
      assert.equal(init?.method, "GET");
      requests += 1;
      assert.ok(init?.signal);
      return new Promise((_resolve, reject) => {
        const unexpected = setTimeout(() => reject(new Error("GET failed to abort")), 1_000);
        init.signal!.addEventListener(
          "abort",
          () => {
            clearTimeout(unexpected);
            aborted = true;
            time = 1_005;
            reject(init.signal!.reason);
          },
          { once: true },
        );
      });
    },
  });
  transport.setPendingDeadline(1_005);
  await assert.rejects(transport.api.read("actions/runs"), /original Q transport deadline expired/);
  assert.equal(aborted, true);
  assert.equal(requests, 1);
  assert.deepEqual(sleeps, []);
});

test("a verification transport has no inferred or renewed pending-Q deadline", async () => {
  let requests = 0;
  const sleeps: number[] = [];
  const transport = createIntegrationReadApi({
    ...identity,
    now: () => 1_000_000,
    sleep: async (milliseconds) => {
      sleeps.push(milliseconds);
    },
    fetchImpl: async (_url, init) => {
      assert.equal(init?.method, "GET");
      requests += 1;
      return requests === 1
        ? Response.json({}, { status: 503 })
        : Response.json({ status: "completed", conclusion: "success" });
    },
  });
  assert.deepEqual(await transport.api.read("actions/runs/7"), {
    status: "completed",
    conclusion: "success",
  });
  assert.equal(requests, 2);
  assert.deepEqual(sleeps, [1_000]);
  assert.deepEqual(Object.keys(transport.api).sort(), ["pages", "read"]);
});
