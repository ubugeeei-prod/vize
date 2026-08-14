import assert from "node:assert/strict";
import { test } from "node:test";

import {
  HOST_TEST_COMMAND_ENVIRONMENT_FLAG,
  HOST_TEST_COMPLETION_COMMAND,
  bindHostTestCommands,
  createHostTestCommands,
  type HostTestLanguageClient,
  type TestCompletionRequest,
} from "../../editors/vscode/src/extension-core.ts";
import {
  HOST_TEST_COMPLETION_COMMAND as suiteHostCompletionCommand,
  assertRealHostCompletionLabels,
} from "../../editors/vscode/test/real-host-completion-oracle.mjs";

test("the hidden host completion command only exists for the host smoke", async () => {
  const client = createFakeClient([{ label: "label" }]);
  assert.deepEqual(createHostTestCommands({ environment: {}, getClient: () => client }), []);
  assert.deepEqual(
    createHostTestCommands({
      environment: { VIZE_TEST_ENABLE_HOST_COMMANDS: "0" },
      getClient: () => client,
    }),
    [],
  );

  const commands = createHostTestCommands({
    environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
    getClient: () => undefined,
  });
  assert.deepEqual(
    commands.map((command) => command.command),
    [HOST_TEST_COMPLETION_COMMAND],
  );
  await assert.rejects(
    commands[0].handler({ character: 8, line: 3, uri: "file:///App.vue" }),
    /requires an active language client/,
  );
});

test("the host completion command forwards the request to the Vize language client", async () => {
  const client = createFakeClient([{ label: "label" }]);
  const [command] = createHostTestCommands({
    environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
    getClient: () => client,
  });

  const response = await command.handler({ character: 8, line: 3, uri: "file:///App.vue" });

  assert.deepEqual(client.requests, [
    [
      "textDocument/completion",
      { position: { character: 8, line: 3 }, textDocument: { uri: "file:///App.vue" } },
    ],
  ]);
  assert.deepEqual(response, { isIncomplete: false, items: [{ label: "label" }] });

  const invalidRequests: unknown[] = [
    undefined,
    { character: 8, line: 3 },
    { character: 8, line: 3, uri: 42 },
    { character: 8, line: -1, uri: "file:///App.vue" },
    { character: -1, line: 3, uri: "file:///App.vue" },
    { character: 1.5, line: 3, uri: "file:///App.vue" },
  ];
  for (const invalid of invalidRequests) {
    await assert.rejects(
      command.handler(invalid as TestCompletionRequest),
      /Invalid Vize test completion request/,
    );
  }
  assert.equal(client.requests.length, 1);
});

test("registering the gated host commands leaves an executable command behind", async () => {
  const registry = new Map<string, (request: TestCompletionRequest) => Promise<unknown>>();
  const register = (
    command: string,
    handler: (request: TestCompletionRequest) => Promise<unknown>,
  ) => {
    registry.set(command, handler);
    return { dispose: () => registry.delete(command) };
  };
  const client = createFakeClient([{ label: "label" }]);

  assert.deepEqual(
    bindHostTestCommands({ environment: {}, getClient: () => client, register }),
    [],
  );
  assert.deepEqual([...registry.keys()], []);

  const registrations = bindHostTestCommands({
    environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
    getClient: () => client,
    register,
  });
  assert.deepEqual([...registry.keys()], [HOST_TEST_COMPLETION_COMMAND]);

  const execute = registry.get(HOST_TEST_COMPLETION_COMMAND);
  assert.ok(execute);
  assert.deepEqual(await execute({ character: 8, line: 3, uri: "file:///App.vue" }), {
    isIncomplete: false,
    items: [{ label: "label" }],
  });
  assert.equal(client.requests.length, 1);

  for (const registration of registrations) {
    registration.dispose();
  }
  assert.deepEqual([...registry.keys()], []);
});

test("the real host completion oracle keeps the smoke on the Vize server answer", () => {
  assert.equal(HOST_TEST_COMPLETION_COMMAND, suiteHostCompletionCommand);

  const serverItems = [
    { label: "Child" },
    { label: { label: "amount" } },
    { label: "label" },
    { label: "count" },
  ];
  assert.deepEqual(assertRealHostCompletionLabels({ items: serverItems }), [
    "Child",
    "amount",
    "label",
    "count",
  ]);
  assert.deepEqual(assertRealHostCompletionLabels(serverItems), [
    "Child",
    "amount",
    "label",
    "count",
  ]);

  // An empty or missing answer is what a broken command gate or a dropped
  // client request produces, and the VS Code directive provider is what
  // answering through the wrong provider produces.
  assert.throws(() => assertRealHostCompletionLabels(null), /must include script binding "Child"/);
  assert.throws(
    () => assertRealHostCompletionLabels({ items: [] }),
    /must include script binding "Child"/,
  );
  assert.throws(
    () => assertRealHostCompletionLabels({ items: [...serverItems, { label: "v-if" }] }),
    /must not surface "v-if"/,
  );
});

function createFakeClient(
  items: unknown[],
): HostTestLanguageClient & { requests: [string, unknown][] } {
  const requests: [string, unknown][] = [];
  return {
    requests,
    sendRequest: async (method, params) => {
      requests.push([method, params]);
      return { isIncomplete: false, items };
    },
  };
}
