import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { stripTypeScriptTypes } from "node:module";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const source = fs.readFileSync(path.join(root, "editors/vscode/src/workspace-config.ts"), "utf8");
const body = stripTypeScriptTypes(source, { mode: "transform" })
  .replace(/^import[\s\S]*?;\n/gm, "")
  .replace("export class WorkspaceLspConfigDiscovery", "class WorkspaceLspConfigDiscovery");

const CONFIG_GLOB = "original-project-config-glob";

function fileUri(pathname) {
  return { scheme: "file", path: pathname, fsPath: pathname, toString: () => `file://${pathname}` };
}

function workspaceFolder(pathname, index = 0) {
  return { uri: fileUri(pathname), name: path.basename(pathname), index };
}

function createDiscovery(findFiles = async () => [], initialFolders = ["/workspace"]) {
  const operations = [];
  const trace = [];
  const callbacks = {};
  const watchers = [];
  const disposable = (name) => ({ dispose: () => operations.push(["dispose", name]) });
  const workspace = {
    workspaceFolders: initialFolders.map(workspaceFolder),
    getWorkspaceFolder: (uri) =>
      workspace.workspaceFolders
        .filter(
          ({ uri: folder }) => uri.path === folder.path || uri.path.startsWith(`${folder.path}/`),
        )
        .sort((left, right) => right.uri.path.length - left.uri.path.length)[0],
    createFileSystemWatcher: (glob) => {
      operations.push(["watch", glob]);
      const index = watchers.length;
      const prefix = index === 0 ? "" : `${index}:`;
      const handlers = index === 0 ? callbacks : {};
      const watcher = {
        glob,
        callbacks: handlers,
        dispose: () => operations.push(["dispose", `${prefix}watcher`]),
        onDidCreate: (callback) => {
          handlers.create = callback;
          return disposable(`${prefix}create`);
        },
        onDidChange: (callback) => {
          handlers.change = callback;
          return disposable(`${prefix}change`);
        },
        onDidDelete: (callback) => {
          handlers.delete = callback;
          return disposable(`${prefix}delete`);
        },
      };
      watchers.push(watcher);
      return watcher;
    },
    onDidChangeWorkspaceFolders: (callback) => {
      callbacks.folders = callback;
      return disposable("folders");
    },
    findFiles: (...arguments_) => {
      operations.push(["findFiles", ...arguments_]);
      return findFiles();
    },
  };
  class RelativePattern {
    constructor(baseUri, pattern) {
      this.baseUri = baseUri.uri ?? baseUri;
      this.pattern = pattern;
    }
  }
  const context = vm.createContext({
    fs,
    path,
    workspace,
    Uri: { file: fileUri },
    RelativePattern,
    WORKSPACE_PROJECT_CONFIG_FILES: [],
    WORKSPACE_PROJECT_CONFIG_GLOB: CONFIG_GLOB,
    describeCapabilities: () => "original description",
    trace,
  });
  const discovery = new vm.Script(
    `${body}\nnew WorkspaceLspConfigDiscovery([], message => trace.push(message));`,
  ).runInContext(context);
  const setFolders = (paths) => {
    const previous = workspace.workspaceFolders;
    const next = paths.map(workspaceFolder);
    workspace.workspaceFolders = next;
    callbacks.folders({
      added: next.filter((folder) => !previous.some((old) => old.uri.path === folder.uri.path)),
      removed: previous.filter((folder) => !next.some((item) => item.uri.path === folder.uri.path)),
    });
  };
  return { discovery, operations, callbacks, trace, watchers, setFolders };
}

await test("actual watcher traces original accepted/rejected create/change/delete callbacks and preserves all subscriptions", () => {
  const { discovery, callbacks, operations, trace } = createDiscovery();
  const reasons = [];
  const subscriptions = discovery.watch((reason) => reasons.push(reason));
  for (const kind of ["create", "change", "delete"]) {
    for (const pathname of [
      "/workspace/packages/app/tsconfig.app.json",
      "/workspace/node_modules/tsconfig.json",
      "/workspace/target/tsconfig.json",
      "/workspace/dist/tsconfig.json",
      "/workspace/build/tsconfig.json",
      "/workspace/.git/tsconfig.json",
    ]) {
      callbacks[kind](fileUri(pathname));
    }
  }
  callbacks.folders({ added: [], removed: [] });
  assert.deepEqual(reasons, [
    "project configuration changed",
    "project configuration changed",
    "project configuration changed",
    "workspace changed",
  ]);
  assert.equal(trace.length, 19);
  for (const kind of ["create", "change", "delete"]) {
    assert.ok(
      trace.includes(
        `watch callback kind=${kind} accepted=true uri=file:///workspace/packages/app/tsconfig.app.json`,
      ),
    );
    assert.ok(
      trace.includes(
        `watch callback kind=${kind} accepted=false uri=file:///workspace/node_modules/tsconfig.json`,
      ),
    );
  }
  for (const subscription of subscriptions) subscription.dispose();
  assert.equal(operations[0][0], "watch");
  assert.equal(operations[0][1].baseUri.fsPath, "/workspace");
  assert.equal(operations[0][1].pattern, CONFIG_GLOB);
  assert.deepEqual(
    operations
      .slice(1)
      .map((operation) => operation.join(":"))
      .sort((left, right) => left.localeCompare(right)),
    [
      "dispose:watcher",
      "dispose:create",
      "dispose:change",
      "dispose:delete",
      "dispose:folders",
    ].sort((left, right) => left.localeCompare(right)),
  );
});

await test("actual findFiles keeps original arguments, commits only completed result, and rethrows identical rejection", async () => {
  let resolve;
  const pending = new Promise((complete) => {
    resolve = complete;
  });
  const { discovery, operations, trace } = createDiscovery(() => pending);
  const refresh = discovery.refresh(7);
  assert.equal(discovery.hasConfig, false);
  assert.deepEqual(trace, ["findFiles enter generation=7"]);
  resolve([fileUri("/workspace/packages/app/tsconfig.app.json")]);
  await refresh;
  assert.equal(discovery.hasConfig, true);
  assert.deepEqual(operations, [
    ["findFiles", "original-project-config-glob", "**/{node_modules,.git,target,dist,build}/**", 1],
  ]);
  assert.deepEqual(trace, [
    "findFiles enter generation=7",
    "findFiles return generation=7 discovered=true",
  ]);
  const originalError = new Error("original search refusal");
  const refusal = createDiscovery(async () => {
    throw originalError;
  });
  await assert.rejects(refusal.discovery.refresh(8), (error) => error === originalError);
  assert.equal(refusal.discovery.hasConfig, false);
  assert.deepEqual(refusal.trace, ["findFiles enter generation=8", "findFiles error generation=8"]);
});

for (const [olderResult, latestResult] of [
  [[fileUri("/workspace/packages/deleted/tsconfig.json")], []],
  [[], [fileUri("/workspace/packages/created/tsconfig.json")]],
]) {
  await test(`out-of-order nested discovery retains the latest ${latestResult.length ? "creation" : "deletion"}`, async () => {
    const searches = [];
    const { discovery, trace } = createDiscovery(
      () => new Promise((resolve) => searches.push(resolve)),
    );
    const older = discovery.refresh();
    const latest = discovery.refresh();
    searches[1](latestResult);
    await latest;
    assert.equal(discovery.hasConfig, latestResult.length > 0);
    searches[0](olderResult);
    await older;
    assert.equal(discovery.hasConfig, latestResult.length > 0);
    assert.equal(trace.at(-1), "findFiles refused generation=undefined stage=superseded");
  });
}

await test("each workspace folder owns an explicit recursive watcher and independent configuration callbacks", () => {
  const { discovery, watchers, operations, trace } = createDiscovery(undefined, [
    "/workspace/one",
    "/workspace/two",
  ]);
  const reasons = [];
  const subscriptions = discovery.watch((reason) => reasons.push(reason));
  assert.deepEqual(
    watchers.map(({ glob }) => [glob.baseUri.fsPath, glob.pattern]),
    [
      ["/workspace/one", CONFIG_GLOB],
      ["/workspace/two", CONFIG_GLOB],
    ],
  );
  assert.equal(
    operations.some(([kind, glob]) => kind === "watch" && typeof glob === "string"),
    false,
  );
  for (const watcher of watchers) {
    for (const kind of ["create", "change", "delete"]) {
      watcher.callbacks[kind](
        fileUri(`${watcher.glob.baseUri.fsPath}/packages/app/tsconfig.app.json`),
      );
      watcher.callbacks[kind](fileUri(`${watcher.glob.baseUri.fsPath}/node_modules/tsconfig.json`));
    }
  }
  assert.deepEqual(reasons, Array(6).fill("project configuration changed"));
  assert.equal(trace.filter((message) => message.includes("accepted=true")).length, 6);
  assert.equal(trace.filter((message) => message.includes("accepted=false")).length, 6);
  for (const subscription of subscriptions) subscription.dispose();
});

await test("folder updates retain existing watchers, register additions and retire removed callbacks", () => {
  const { discovery, watchers, operations, setFolders, trace } = createDiscovery();
  const reasons = [];
  const subscriptions = discovery.watch((reason) => reasons.push(reason));
  const original = watchers[0];
  setFolders(["/workspace", "/another"]);
  assert.equal(watchers.length, 2);
  assert.equal(watchers[0], original);
  assert.equal(operations.filter(([kind]) => kind === "dispose").length, 0);
  assert.equal(watchers[1].glob.baseUri.fsPath, "/another");
  setFolders(["/another"]);
  assert.deepEqual(
    operations
      .filter(([kind]) => kind === "dispose")
      .map((operation) => operation.join(":"))
      .sort((left, right) => left.localeCompare(right)),
    ["dispose:watcher", "dispose:create", "dispose:change", "dispose:delete"].sort((left, right) =>
      left.localeCompare(right),
    ),
  );
  reasons.length = 0;
  const traceLength = trace.length;
  for (const kind of ["create", "change", "delete"]) {
    original.callbacks[kind](fileUri("/workspace/packages/app/tsconfig.json"));
  }
  assert.deepEqual(reasons, []);
  assert.equal(
    trace.length,
    traceLength,
    "removed watcher callbacks are closed before tracing or dispatch",
  );
  watchers[1].callbacks.change(fileUri("/another/packages/app/vite.config.ts"));
  assert.deepEqual(reasons, ["project configuration changed"]);
  setFolders(["/workspace", "/another"]);
  assert.equal(watchers.length, 3, "a re-added folder receives fresh callback ownership");
  reasons.length = 0;
  original.callbacks.change(fileUri("/workspace/tsconfig.json"));
  watchers[2].callbacks.change(fileUri("/workspace/tsconfig.json"));
  assert.deepEqual(reasons, ["project configuration changed"]);
  for (const subscription of subscriptions) subscription.dispose();
});

await test("overlapping folders route a nested config event to exactly its most specific active owner", () => {
  const { discovery, watchers, setFolders } = createDiscovery(undefined, [
    "/workspace",
    "/workspace/packages/app",
  ]);
  const reasons = [];
  const subscriptions = discovery.watch((reason) => reasons.push(reason));
  const nestedConfig = fileUri("/workspace/packages/app/tsconfig.app.json");
  for (const kind of ["create", "change", "delete"]) {
    watchers[0].callbacks[kind](nestedConfig);
    watchers[1].callbacks[kind](nestedConfig);
  }
  assert.deepEqual(reasons, Array(3).fill("project configuration changed"));
  setFolders(["/workspace"]);
  reasons.length = 0;
  watchers[0].callbacks.change(nestedConfig);
  watchers[1].callbacks.change(nestedConfig);
  assert.deepEqual(
    reasons,
    ["project configuration changed"],
    "the surviving parent resumes ownership",
  );
  for (const subscription of subscriptions) subscription.dispose();
});

await test("empty workspaces register no global fallback and begin watching newly added folders", () => {
  const { discovery, watchers, setFolders } = createDiscovery(undefined, []);
  const reasons = [];
  const subscriptions = discovery.watch((reason) => reasons.push(reason));
  assert.equal(watchers.length, 0);
  setFolders(["/workspace"]);
  assert.equal(watchers.length, 1);
  assert.equal(watchers[0].glob.baseUri.fsPath, "/workspace");
  assert.equal(watchers[0].glob.pattern, CONFIG_GLOB);
  reasons.length = 0;
  watchers[0].callbacks.create(fileUri("/workspace/packages/app/vite.config.ts"));
  assert.deepEqual(reasons, ["project configuration changed"]);
  for (const subscription of subscriptions) subscription.dispose();
});

await test("disposal closes every watcher and ignores retained workspace-folder and file callbacks", () => {
  const { discovery, watchers, operations, setFolders, trace } = createDiscovery(undefined, [
    "/workspace",
    "/another",
  ]);
  const reasons = [];
  const subscriptions = discovery.watch((reason) => reasons.push(reason));
  for (const subscription of subscriptions) subscription.dispose();
  assert.deepEqual(
    operations
      .filter(([kind]) => kind === "dispose")
      .map((operation) => operation[1])
      .sort((left, right) => left.localeCompare(right)),
    [
      "watcher",
      "create",
      "change",
      "delete",
      "1:watcher",
      "1:create",
      "1:change",
      "1:delete",
      "folders",
    ].sort((left, right) => left.localeCompare(right)),
  );
  const traceLength = trace.length;
  const operationsLength = operations.length;
  setFolders(["/new"]);
  for (const watcher of watchers) {
    for (const kind of ["create", "change", "delete"]) {
      watcher.callbacks[kind](fileUri(`${watcher.glob.baseUri.fsPath}/tsconfig.json`));
    }
  }
  assert.equal(watchers.length, 2);
  assert.equal(operations.length, operationsLength);
  assert.equal(trace.length, traceLength);
  assert.deepEqual(reasons, []);
});
