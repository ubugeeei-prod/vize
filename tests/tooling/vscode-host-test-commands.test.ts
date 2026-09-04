import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import {
  HOST_TEST_COMMAND_ENVIRONMENT_FLAG,
  HOST_TEST_COMPLETION_COMMAND,
  HOST_TEST_SERVER_INFO_COMMAND,
  bindHostTestCommands,
  createHostTestCommands,
  type HostTestLanguageClient,
  type HostTestServerInfo,
  type TestCompletionRequest,
} from "../../editors/vscode/src/extension-core.ts";
import {
  HOST_TEST_COMPLETION_COMMAND as suiteHostCompletionCommand,
  assertRealHostCompletionLabels,
} from "../../editors/vscode/test/real-host-completion-oracle.mjs";
import {
  HOST_TEST_SERVER_INFO_COMMAND as suiteHostServerInfoCommand,
  assertRealHostServerInfo,
} from "../../editors/vscode/test/real-host-server-info-oracle.mjs";

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
  assertExactCommandMembership(
    commands.map((command) => command.command),
    [HOST_TEST_COMPLETION_COMMAND, HOST_TEST_SERVER_INFO_COMMAND],
  );
  const completionCommand = findHostCommand(commands, HOST_TEST_COMPLETION_COMMAND);
  const serverInfoCommand = findHostCommand(commands, HOST_TEST_SERVER_INFO_COMMAND);
  await assert.rejects(
    completionCommand.handler({ character: 8, line: 3, uri: "file:///App.vue" }),
    /requires an active language client/,
  );
  await assert.rejects(serverInfoCommand.handler(), /requires selected server evidence/);
});

test("the host completion command forwards the request to the Vize language client", async () => {
  const client = createFakeClient([{ label: "label" }]);
  const commands = createHostTestCommands({
    environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
    getClient: () => client,
  });
  const command = findHostCommand(commands, HOST_TEST_COMPLETION_COMMAND);

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

test("the host server info command returns the selected server evidence", async () => {
  const serverInfo = {
    extensionVersion: "0.392.0",
    path: "/repo/target/ci/vize",
    source: "configured",
    status: "ready",
    version: "0.392.0",
  } satisfies HostTestServerInfo;
  const commands = createHostTestCommands({
    environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
    getClient: () => createFakeClient([]),
    getServerInfo: () => serverInfo,
  });

  const execute = commands.find((command) => command.command === HOST_TEST_SERVER_INFO_COMMAND);

  assert.ok(execute);
  assert.deepEqual(await execute.handler(), serverInfo);
});

test("registering the gated host commands leaves an executable command behind", async () => {
  const registry = new Map<string, (request?: unknown) => Promise<unknown>>();
  const registerCalls: string[] = [];
  const register = (command: string, handler: (request?: unknown) => Promise<unknown>) => {
    registerCalls.push(command);
    registry.set(command, handler);
    return { dispose: () => registry.delete(command) };
  };
  const client = createFakeClient([{ label: "label" }]);

  assert.deepEqual(
    bindHostTestCommands({ environment: {}, getClient: () => client, register }),
    [],
  );
  assert.deepEqual([...registry.keys()], []);
  assert.deepEqual(registerCalls, []);

  const registrations = bindHostTestCommands({
    environment: { [HOST_TEST_COMMAND_ENVIRONMENT_FLAG]: "1" },
    getClient: () => client,
    register,
  });
  assertExactCommandMembership(registerCalls, [
    HOST_TEST_COMPLETION_COMMAND,
    HOST_TEST_SERVER_INFO_COMMAND,
  ]);
  assertExactCommandMembership(registry.keys(), [
    HOST_TEST_COMPLETION_COMMAND,
    HOST_TEST_SERVER_INFO_COMMAND,
  ]);

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

test("the real host server info oracle pins the selected server identity", () => {
  assert.equal(HOST_TEST_SERVER_INFO_COMMAND, suiteHostServerInfoCommand);
  const serverPath = fs.realpathSync(process.execPath);

  assert.deepEqual(
    assertRealHostServerInfo(
      {
        extensionVersion: "0.392.0",
        path: serverPath,
        source: "configured",
        status: "ready",
        version: "0.392.0",
      },
      {
        extensionVersion: "0.392.0",
        serverPath,
        serverVersion: "0.392.0",
      },
    ),
    {
      extensionVersion: "0.392.0",
      path: serverPath,
      source: "configured",
      status: "ready",
      version: "0.392.0",
    },
  );
  assert.throws(() =>
    assertRealHostServerInfo(
      {
        extensionVersion: "0.392.0",
        path: serverPath,
        source: "configured",
        status: "ready",
        version: "0.391.0",
      },
      {
        extensionVersion: "0.392.0",
        serverPath,
        serverVersion: "0.392.0",
      },
    ),
  );
});

function findHostCommand(
  commands: ReturnType<typeof createHostTestCommands>,
  commandId: string,
): ReturnType<typeof createHostTestCommands>[number] {
  const command = commands.find((candidate) => candidate.command === commandId);
  assert.ok(command, `${commandId} must be registered`);
  return command;
}

function assertExactCommandMembership(actual: Iterable<string>, expected: readonly string[]) {
  assert.deepEqual([...actual].sort(), [...expected].sort());
}

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
