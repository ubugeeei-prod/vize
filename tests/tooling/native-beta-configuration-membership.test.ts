import assert from "node:assert/strict";
import { test } from "node:test";
import { storageFixture } from "./support/native-beta-storage-fixture.ts";
import {
  verifyAfterAttachedStorage,
  type AttachedOwner,
} from "./support/native-beta-dto/beta-attached-route.ts";
import { verifyAttachedConfigurationMembership } from "./support/native-beta-dto/configuration-membership.ts";
import type { WatcherContext } from "./support/native-beta-dto/stable-watcher-boundary.ts";
import path from "node:path";

type Packet = Record<string, unknown>;
// Synthetic contracts only; the authenticated failed e754 raw replay is separate.
function fixture(withVirtual = false) {
  const input = storageFixture(),
    root = input.root,
    authored = input.attached.root;
  const source = "e".repeat(40),
    config = root + "/tsconfig.json";
  const context: WatcherContext = {
    role: "after",
    source,
    sdkVersion: "7.1.0-beta",
    sdkSha256: "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
    sdkExecutable: "/sdk/node_modules/@typescript/typescript-linux-x64/lib/tsc",
    projectRoots: [authored],
  };
  const authoredConfig = {
    compilerOptions: {
      strict: true,
      target: "ESNext",
      module: "ESNext",
      moduleResolution: "Bundler",
      types: [],
      noEmit: true,
    },
    files: ["main.ts", "support.ts"],
  };
  const attached = { ...input.attached, config: JSON.stringify(authoredConfig) };
  const mapped = [
    ...(withVirtual ? [root + "/Virtual.vue.ts"] : []),
    root + "/main.ts",
    root + "/support.ts",
  ];
  const parsed = {
    fileNames: [authored + "/main.ts", authored + "/support.ts", ...mapped],
    options: {
      allowImportingTsExtensions: true,
      module: 99,
      moduleResolution: 100,
      noEmit: true,
      strict: true,
      target: 99,
      types: [],
      configFilePath: config,
    },
    compileOnSave: false,
    raw: {
      ...JSON.parse(attached.materializedBatch.storageAuthority.config.text),
      files: ["main.ts", "support.ts"].map((file) =>
        path.posix.relative(root, authored + "/" + file),
      ),
    },
    errors: [],
  };
  const project = () => ({
    id: config,
    configFileName: config,
    currentDirectory: root,
    dirty: false,
    parsedCommandLine: structuredClone(parsed),
  });
  const owner: AttachedOwner = {
    nativeOwner: 123,
    client: [
      {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: { rootPath: root, rootUri: "file://" + root },
      },
      {
        jsonrpc: "2.0",
        id: 2,
        method: "custom/projectInfo",
        params: { textDocument: { uri: "file://" + mapped[0] } },
      },
    ],
    server: [
      {
        jsonrpc: "2.0",
        id: 1,
        result: { serverInfo: { name: "typescript", version: "7.1.0-beta" } },
      },
      { jsonrpc: "2.0", id: 2, result: { configFilePath: config } },
    ],
    apiClient: [
      { jsonrpc: "2.0", id: 1, method: "initialize", params: {} },
      { jsonrpc: "2.0", id: 2, method: "parseConfigFile", params: { file: config } },
      {
        jsonrpc: "2.0",
        id: 3,
        method: "createSnapshot",
        params: { ensurePrograms: true, openProjects: [config] },
      },
      {
        jsonrpc: "2.0",
        id: 4,
        method: "updateSnapshot",
        params: { snapshot: 2, changes: { ensurePrograms: true, openProjects: [config] } },
      },
      {
        jsonrpc: "2.0",
        id: 5,
        method: "getDefaultProjectForFile",
        params: { file: mapped[0], snapshot: 3 },
      },
      { jsonrpc: "2.0", id: 6, method: "describeCapabilities", params: {} },
      { jsonrpc: "2.0", id: 7, method: "release", params: { snapshot: 2 } },
      {
        jsonrpc: "2.0",
        id: 8,
        method: "updateSnapshot",
        params: {
          snapshot: 3,
          changes: {
            ensurePrograms: true,
            openProjects: [config],
            fileNotifications: { changed: [root + "/support.ts"] },
          },
        },
      },
      {
        jsonrpc: "2.0",
        id: 9,
        method: "getDefaultProjectForFile",
        params: { file: mapped[0], snapshot: 4 },
      },
    ],
    apiServer: [
      { jsonrpc: "2.0", id: 1, result: {} },
      { jsonrpc: "2.0", id: 2, result: parsed },
      { jsonrpc: "2.0", id: 3, result: { snapshot: 2, projects: [project()], operation: {} } },
      { jsonrpc: "2.0", id: 4, result: { snapshot: 3, projects: [], operation: {} } },
      { jsonrpc: "2.0", id: 5, result: project() },
      {
        jsonrpc: "2.0",
        id: 6,
        error: {
          code: -32603,
          message: 'api: invalid request: unknown API method "describeCapabilities"',
        },
      },
      { jsonrpc: "2.0", id: 7, result: null },
      { jsonrpc: "2.0", id: 8, result: { snapshot: 4, projects: [], operation: {} } },
      { jsonrpc: "2.0", id: 9, result: project() },
    ],
  };
  if (withVirtual) {
    owner.client = [
      ...owner.client,
      {
        jsonrpc: "2.0",
        method: "textDocument/didOpen",
        params: {
          textDocument: {
            uri: "file://" + mapped[0],
            text: attached.materializedBatch.source,
            languageId: "typescript",
            version: 1,
          },
        },
      },
      {
        jsonrpc: "2.0",
        id: 3,
        method: "textDocument/diagnostic",
        params: { textDocument: { uri: "file://" + mapped[0] } },
      },
    ];
    owner.server = [
      ...owner.server,
      { jsonrpc: "2.0", id: 3, result: { kind: "full", items: [] } },
    ];
  }
  const storage = verifyAfterAttachedStorage([owner], context, source, attached);
  return { owner, context, source, storage, attached, parsed };
}
const object = (value: unknown) => value as Packet;
const request = (f: ReturnType<typeof fixture>, id: number) =>
  object(f.owner.apiClient.find((row) => object(row).id === id));
const reply = (f: ReturnType<typeof fixture>, id: number) =>
  object(f.owner.apiServer.find((row) => object(row).id === id));
const verify = (f: ReturnType<typeof fixture>) =>
  verifyAttachedConfigurationMembership(f.owner, f.context, f.source, f.storage, f.attached);

test("owned main and virtual transitions retain whole configuration membership pairs without semantic credit", () => {
  for (const withVirtual of [false, true]) {
    const f = fixture(withVirtual),
      result = verify(f);
    assert.equal(result.length, 2);
    assert.deepEqual(
      result.map((row) => row.request),
      [request(f, 5), request(f, 9)],
    );
    assert.deepEqual(
      result.map((row) => row.response),
      [reply(f, 5), reply(f, 9)],
    );
    assert.deepEqual(result[0]!.parsedConfiguration, f.parsed);
    assert.deepEqual(reply(f, 6).error, {
      code: -32603,
      message: 'api: invalid request: unknown API method "describeCapabilities"',
    });
    assert.equal("semanticPass" in result[0]!, false);
  }
});
test("an initial owner with no membership RPC retains the existing six-method route", () => {
  const f = fixture();
  f.owner.apiClient = f.owner.apiClient.filter(
    (row) => object(row).method !== "getDefaultProjectForFile",
  );
  f.owner.apiServer = f.owner.apiServer.filter((row) => ![5, 9].includes(Number(object(row).id)));
  assert.deepEqual(verify(f), []);
});
test("wrong source, image, owner, member, configuration, reply or snapshot chain refuses", () => {
  type F = ReturnType<typeof fixture>;
  const mutations: Array<[string, (f: F) => void]> = [
    [
      "before",
      (f) => {
        f.context.role = "before";
      },
    ],
    [
      "source",
      (f) => {
        f.context.source = "0".repeat(40);
      },
    ],
    [
      "stable",
      (f) => {
        f.context.sdkVersion = "7.0.2";
      },
    ],
    [
      "image",
      (f) => {
        f.context.sdkSha256 = "0".repeat(64);
      },
    ],
    [
      "authored owner",
      (f) => {
        object(object(f.owner.client[0]).params).rootPath = f.storage.authoredRoot;
      },
    ],
    [
      "foreign root",
      (f) => {
        f.context.projectRoots = ["/tmp/foreign"];
      },
    ],
    [
      "wrong parse config",
      (f) => {
        object(request(f, 2).params).file = "/tmp/foreign/tsconfig.json";
      },
    ],
    [
      "extra parse request field",
      (f) => {
        object(request(f, 2).params).extra = true;
      },
    ],
    [
      "wrong parsed roster",
      (f) => {
        f.parsed.fileNames.push(f.storage.materializedRoot + "/outside.ts");
      },
    ],
    [
      "wrong compiler option",
      (f) => {
        f.parsed.options.strict = false;
      },
    ],
    [
      "parsed error",
      (f) => {
        Object.assign(f.parsed, { errors: [{}] });
      },
    ],
    [
      "wrong source config",
      (f) => {
        f.attached.config = "{}";
      },
    ],
    [
      "unmapped member",
      (f) => {
        object(request(f, 5).params).file = f.storage.authoredRoot + "/main.ts";
      },
    ],
    [
      "nonpreferred support",
      (f) => {
        object(request(f, 5).params).file = f.storage.materializedRoot + "/support.ts";
      },
    ],
    [
      "wrong live snapshot",
      (f) => {
        object(request(f, 5).params).snapshot = 2;
      },
    ],
    [
      "extra membership field",
      (f) => {
        object(request(f, 5).params).project = "invented";
      },
    ],
    [
      "wrong update parent",
      (f) => {
        object(request(f, 8).params).snapshot = 2;
      },
    ],
    [
      "wrong update config",
      (f) => {
        object(object(request(f, 8).params).changes).openProjects = ["/tmp/foreign/tsconfig.json"];
      },
    ],
    [
      "foreign file notification",
      (f) => {
        object(object(request(f, 8).params).changes).fileNotifications = {
          changed: ["/tmp/foreign.ts"],
        };
      },
    ],
    [
      "reused snapshot",
      (f) => {
        object(reply(f, 8).result).snapshot = 3;
        object(request(f, 9).params).snapshot = 3;
      },
    ],
    [
      "snapshot has error",
      (f) => {
        Object.assign(reply(f, 4), { error: { code: -1 } });
      },
    ],
    [
      "null project",
      (f) => {
        reply(f, 5).result = null;
      },
    ],
    [
      "project has error",
      (f) => {
        Object.assign(reply(f, 5), { error: { code: -1 } });
      },
    ],
    [
      "wrong returned config",
      (f) => {
        object(reply(f, 5).result).configFileName = "/tmp/foreign/tsconfig.json";
      },
    ],
    [
      "wrong project id",
      (f) => {
        object(reply(f, 5).result).id = "invented";
      },
    ],
    [
      "missing member",
      (f) => {
        object(reply(f, 5).result).parsedCommandLine = { ...f.parsed, fileNames: [] };
      },
    ],
    [
      "extra result field",
      (f) => {
        object(reply(f, 5).result).extra = true;
      },
    ],
    [
      "wrong startup member",
      (f) => {
        object(object(object(f.owner.client[1]).params).textDocument).uri =
          "file://" + f.storage.materializedRoot + "/support.ts";
      },
    ],
    [
      "duplicate membership",
      (f) => {
        f.owner.apiClient = [...f.owner.apiClient, { ...request(f, 9), id: 10 }];
        f.owner.apiServer = [...f.owner.apiServer, { ...reply(f, 9), id: 10 }];
      },
    ],
    [
      "reversed snapshot reply",
      (f) => {
        const responses = [...f.owner.apiServer];
        [responses[3], responses[4]] = [responses[4]!, responses[3]!];
        f.owner.apiServer = responses;
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const f = fixture();
    mutate(f);
    assert.throws(() => verify(f), name);
  }
});
test("membership before snapshot creation or after release cannot borrow a known handle", () => {
  const first = fixture();
  first.owner.apiClient = [
    request(first, 5),
    ...first.owner.apiClient.filter((row) => object(row).id !== 5),
  ];
  assert.throws(() => verify(first));
  const released = fixture();
  released.owner.apiClient = [
    ...released.owner.apiClient,
    { jsonrpc: "2.0", id: 10, method: "release", params: { snapshot: 4 } },
    { ...request(released, 9), id: 11 },
  ];
  released.owner.apiServer = [
    ...released.owner.apiServer,
    { jsonrpc: "2.0", id: 10, result: null },
    { ...reply(released, 9), id: 11 },
  ];
  assert.throws(() => verify(released));
});

test("a server roster cannot erase the independently observed Virtual client phase", () => {
  const f = fixture(true),
    root = f.storage.materializedRoot;
  const main = root + "/main.ts";
  f.parsed.fileNames = f.parsed.fileNames.filter((name) => name !== root + "/Virtual.vue.ts");
  for (const id of [5, 9]) {
    object(request(f, id).params).file = main;
    object(reply(f, id).result).parsedCommandLine = structuredClone(f.parsed);
  }
  object(object(object(f.owner.client[1]).params).textDocument).uri = "file://" + main;
  assert.throws(() => verify(f));
  const openedOnly = fixture(true);
  openedOnly.owner.client = openedOnly.owner.client.filter(
    (row) => object(row).method !== "textDocument/diagnostic",
  );
  assert.throws(() => verify(openedOnly));
  const queriedOnly = fixture(true);
  queriedOnly.owner.client = queriedOnly.owner.client.filter(
    (row) => object(row).method !== "textDocument/didOpen",
  );
  assert.throws(() => verify(queriedOnly));
  const wrongBytes = fixture(true);
  const opened = object(
    wrongBytes.owner.client.find((row) => object(row).method === "textDocument/didOpen"),
  );
  object(object(opened.params).textDocument).text = "invented";
  assert.throws(() => verify(wrongBytes));
});
