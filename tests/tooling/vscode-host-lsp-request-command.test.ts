import assert from "node:assert/strict";
import { test } from "node:test";

import {
  HOST_TEST_COMMAND_ENVIRONMENT_FLAG,
  HOST_TEST_LSP_REQUEST_COMMAND,
  createHostTestCommands,
  type HostTestLanguageClient,
  type TestLspRequest,
} from "../../editors/vscode/src/host-test-core.ts";
import {
  HOST_TEST_LSP_REQUEST_COMMAND as suiteHostLspRequestCommand,
  assertRealHostTemplateBindingHover,
} from "../../editors/vscode/test/real-host-lsp-request-oracle.mjs";

test("the generic host LSP request command only exists for the host smoke", () => {
  const client = createFakeClientResponse(null);

  assert.equal(hasLspRequestCommand({ environment: {}, getClient: () => client }), false);
  assert.equal(
    hasLspRequestCommand({
      environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "0" },
      getClient: () => client,
    }),
    false,
  );
  assert.equal(
    hasLspRequestCommand({
      environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
      getClient: () => client,
    }),
    true,
  );
});

test("the generic host LSP request command forwards method and params", async () => {
  const response = [{ contents: "hover" }];
  const client = createFakeClientResponse(response);
  const command = findLspRequestCommand({
    environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
    getClient: () => client,
  });
  const params = {
    position: { character: 4, line: 2 },
    textDocument: { uri: "file:///App.vue" },
  };

  assert.deepEqual(await command.handler({ method: "textDocument/hover", params }), response);
  assert.deepEqual(client.requests, [["textDocument/hover", params]]);
});

test("the generic host LSP request command waits for the active language client", async () => {
  const response = [{ contents: "hover after startup" }];
  let client: ReturnType<typeof createFakeClientResponse> | undefined;
  const command = findLspRequestCommand({
    environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
    getClient: () => client,
  });
  const request = {
    method: "textDocument/hover",
    params: {
      position: { character: 4, line: 2 },
      textDocument: { uri: "file:///App.vue" },
    },
  };

  await assert.rejects(
    command.handler(request),
    /Vize test LSP request command requires an active language client/,
  );

  client = createFakeClientResponse(response);
  assert.deepEqual(await command.handler(request), response);
  assert.deepEqual(client.requests, [[request.method, request.params]]);
});

test("the generic host LSP request command propagates backend failures", async () => {
  const failure = new Error("backend rejected hover");
  const client = createFakeClientFailure(failure);
  const command = findLspRequestCommand({
    environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
    getClient: () => client,
  });
  const params = {
    position: { character: 4, line: 2 },
    textDocument: { uri: "file:///App.vue" },
  };

  await assert.rejects(command.handler({ method: "textDocument/hover", params }), (error) => {
    assert.strictEqual(error, failure);
    return true;
  });
  assert.deepEqual(client.requests, [["textDocument/hover", params]]);
});

test("the generic host LSP request command validates request shape", async () => {
  const client = createFakeClientResponse(null);
  const command = findLspRequestCommand({
    environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
    getClient: () => client,
  });

  for (const invalid of [undefined, {}, { method: "" }, { method: "   " }, { method: 42 }]) {
    await assert.rejects(
      command.handler(invalid as TestLspRequest),
      /Invalid Vize test LSP request/,
    );
  }
  assert.deepEqual(client.requests, []);
});

test("the real host LSP request oracle keeps the command id in sync", () => {
  assert.equal(HOST_TEST_LSP_REQUEST_COMMAND, suiteHostLspRequestCommand);
});

test("the real host LSP request oracle pins backend typed hover content", () => {
  const hover = {
    contents: {
      kind: "markdown",
      value: ["```typescript", 'const label: "hello from vize"', "```"].join("\n"),
    },
  };

  assert.equal(
    assertRealHostTemplateBindingHover(hover),
    '```typescript\nconst label: "hello from vize"\n```',
  );
  assert.throws(
    () =>
      assertRealHostTemplateBindingHover({
        contents: "Template binding from script: Ref<unknown>",
      }),
    /Template binding from script/,
  );
  assert.throws(() => assertRealHostTemplateBindingHover(null), /must include contents/);
});

function hasLspRequestCommand(behavior: Parameters<typeof createHostTestCommands>[0]): boolean {
  return createHostTestCommands(behavior).some(
    (command) => command.command === HOST_TEST_LSP_REQUEST_COMMAND,
  );
}

function findLspRequestCommand(behavior: Parameters<typeof createHostTestCommands>[0]) {
  const command = createHostTestCommands(behavior).find(
    (candidate) => candidate.command === HOST_TEST_LSP_REQUEST_COMMAND,
  );
  assert.ok(command, `${HOST_TEST_LSP_REQUEST_COMMAND} must be registered`);
  return command;
}

function createFakeClientResponse(
  response: unknown,
): HostTestLanguageClient & { requests: [string, unknown][] } {
  const requests: [string, unknown][] = [];
  return {
    requests,
    sendRequest: async (method, params) => {
      requests.push([method, params]);
      return response;
    },
  };
}

function createFakeClientFailure(
  failure: Error,
): HostTestLanguageClient & { requests: [string, unknown][] } {
  const requests: [string, unknown][] = [];
  return {
    requests,
    sendRequest: async (method, params) => {
      requests.push([method, params]);
      throw failure;
    },
  };
}
