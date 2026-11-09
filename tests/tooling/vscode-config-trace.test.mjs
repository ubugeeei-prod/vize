import assert from "node:assert/strict";
import { test } from "node:test";
import { CONFIG_GLOB, createDiscovery, fileUri } from "./support/vscode-workspace-config.mjs";

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
  assert.equal(trace.length, 38);
  assert.equal(trace[0], `watch register workspace=file:///workspace pattern=${CONFIG_GLOB}`);
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

await test("actual findFiles enumerates configs with original glob/exclusions, commits completed result, and rethrows identical rejection", async () => {
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
    ["findFiles", "original-project-config-glob", "**/{node_modules,.git,target,dist,build}/**"],
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
    traceLength + 3,
    "removed watcher callbacks retain only passive raw events, never dispatch",
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
  assert.equal(trace.length, traceLength + 6);
  assert.ok(
    trace
      .slice(traceLength)
      .every((line) => line.includes("disposed=true active=false owner=undefined")),
  );
  assert.deepEqual(reasons, []);
});

await test("raw watcher events retain URI and lifecycle before the original ownership rejection", () => {
  const { discovery, callbacks, trace } = createDiscovery();
  const reasons = [];
  const subscriptions = discovery.watch((reason) => reasons.push(reason));
  callbacks.change(fileUri("/outside/packages/app/tsconfig.app.json"));
  assert.equal(
    trace.at(-1),
    "watch event kind=change uri=file:///outside/packages/app/tsconfig.app.json workspace=file:///workspace disposed=false active=true owner=undefined",
  );
  assert.deepEqual(reasons, []);
  for (const subscription of subscriptions) subscription.dispose();
  callbacks.change(fileUri("/workspace/packages/app/tsconfig.app.json"));
  assert.equal(
    trace.at(-1),
    "watch event kind=change uri=file:///workspace/packages/app/tsconfig.app.json workspace=file:///workspace disposed=true active=false owner=undefined",
  );
  assert.deepEqual(reasons, []);
});

await test("awaited discovery anchors every known parent before startup and keeps recursive discovery", async () => {
  const first = fileUri("/workspace/packages/app/tsconfig.app.json");
  const sibling = fileUri("/workspace/packages/app/vite.config.ts");
  const second = fileUri("/workspace/packages/other/tsconfig.json");
  let files = [
    first,
    sibling,
    second,
    first,
    fileUri("/workspace/node_modules/tsconfig.json"),
    fileUri("/outside/tsconfig.json"),
  ];
  const { discovery, watchers, nativeWatchers, operations, setFolders } = createDiscovery(
    async () => files,
  );
  const reasons = [];
  const subscriptions = discovery.watch((reason) => reasons.push(reason));
  await discovery.refresh(1);
  assert.deepEqual(
    watchers.map(({ glob }) => [glob.baseUri.fsPath, glob.pattern]),
    [["/workspace", CONFIG_GLOB]],
  );
  assert.deepEqual(
    nativeWatchers.map(({ parent, options }) => [parent, { ...options }]),
    ["/workspace/packages/app", "/workspace/packages/other"].map((parent) => [
      parent,
      { recursive: false, persistent: false, encoding: "utf8" },
    ]),
  );
  const parent = nativeWatchers[0];
  // Both backends may report a known file; its active parent owns dispatch.
  for (const kind of ["create", "change", "delete"]) {
    watchers[0].callbacks[kind](first);
    parent.callback(kind === "change" ? "change" : "rename", "tsconfig.app.json");
  }
  parent.callback("change", "vite.config.ts");
  parent.callback("change", "unrelated.ts");
  parent.callback("change", "../other/tsconfig.json");
  parent.callback("change", "TSCONFIG.APP.JSON");
  assert.deepEqual(reasons, Array(4).fill("project configuration changed"));
  watchers[0].callbacks.create(fileUri("/workspace/new/tsconfig.json"));
  assert.equal(reasons.at(-1), "project configuration changed");
  await discovery.refresh(2);
  assert.equal(nativeWatchers.length, 2, "unchanged parents retain their leases");
  files = [sibling];
  await discovery.refresh(3);
  reasons.length = 0;
  parent.callback("change", "tsconfig.app.json");
  parent.callback("change", "vite.config.ts");
  nativeWatchers[1].callback("change", "tsconfig.json");
  assert.deepEqual(reasons, ["project configuration changed"]);
  assert.equal(
    operations.filter(([kind, index]) => kind === "nativeClose" && index === 1).length,
    1,
  );
  setFolders(["/workspace", "/workspace/packages/app"]);
  const nestedOwner = nativeWatchers.at(-1);
  assert.notEqual(nestedOwner, parent);
  reasons.length = 0;
  parent.callback("change", "vite.config.ts");
  nestedOwner.callback("change", "vite.config.ts");
  assert.deepEqual(reasons, ["project configuration changed"]);
  setFolders(["/workspace"]);
  const activeParent = nativeWatchers.at(-1);
  reasons.length = 0;
  nestedOwner.callback("change", "vite.config.ts");
  activeParent.callback("change", "vite.config.ts");
  assert.deepEqual(reasons, ["project configuration changed"]);
  setFolders([]);
  reasons.length = 0;
  activeParent.callback("change", "vite.config.ts");
  assert.deepEqual(reasons, []);
  setFolders(["/workspace"]);
  const replacement = nativeWatchers.at(-1);
  assert.notEqual(replacement, parent);
  reasons.length = 0;
  parent.callback("change", "vite.config.ts");
  replacement.callback("change", "vite.config.ts");
  assert.deepEqual(reasons, ["project configuration changed"]);
  for (const subscription of subscriptions) subscription.dispose();
  reasons.length = 0;
  replacement.callback("change", "vite.config.ts");
  await discovery.refresh(4);
  assert.deepEqual(reasons, []);
  assert.equal(nativeWatchers.at(-1), replacement, "disposed discovery cannot recreate a lease");
});

await test("superseded and disposed searches cannot install stale parent leases", async () => {
  const searches = [];
  const { discovery, watchers, nativeWatchers } = createDiscovery(
    () => new Promise((resolve) => searches.push(resolve)),
  );
  const subscriptions = discovery.watch(() => {});
  const stale = discovery.refresh(1);
  const current = discovery.refresh(2);
  searches[1]([fileUri("/workspace/current/tsconfig.json")]);
  await current;
  searches[0]([fileUri("/workspace/stale/tsconfig.json")]);
  await stale;
  assert.deepEqual(
    nativeWatchers.map(({ parent }) => parent),
    ["/workspace/current"],
  );
  assert.equal(watchers.length, 1);
  const pending = discovery.refresh(3);
  for (const subscription of subscriptions) subscription.dispose();
  searches[2]([fileUri("/workspace/late/tsconfig.json")]);
  await pending;
  assert.equal(nativeWatchers.length, 1);
});

await test("unattributable and unavailable native watches restore recursive dispatch without restarting", async (t) => {
  for (const failure of ["null", "empty", "error", "close", "throw"]) {
    await t.test(failure, async () => {
      const uri = fileUri("/workspace/app/tsconfig.json");
      const { discovery, watchers, nativeWatchers, trace } = createDiscovery(
        async () => [uri],
        ["/workspace"],
        {
          nativeWatchError: failure === "throw" ? new Error("native watch unavailable") : undefined,
        },
      );
      const reasons = [];
      const subscriptions = discovery.watch((reason) => reasons.push(reason));
      await discovery.refresh(1);
      const native = nativeWatchers[0];
      if (failure === "null") native.callback("change", null);
      if (failure === "empty") native.callback("rename", "");
      if (failure === "error") native.emit("error", new Error("native event failure"));
      if (failure === "close") native.close();
      assert.deepEqual(reasons, [], "unknown filename/error is not evidence of a config change");
      watchers[0].callbacks.change(uri);
      assert.deepEqual(reasons, ["project configuration changed"]);
      if (native) {
        assert.equal(native.closed, true);
        native.callback("change", "tsconfig.json");
        native.emit("error", new Error("late error after close"));
        assert.equal(reasons.length, 1);
      }
      assert.equal(
        trace.some((line) => line.includes("watch native")),
        true,
      );
      for (const subscription of subscriptions) subscription.dispose();
    });
  }
});

await test("custom filesystem configurations keep recursive delivery without native path guessing", async () => {
  const folder = fileUri("/workspace", "vscode-remote");
  const uri = fileUri("/workspace/app/tsconfig.json", "vscode-remote");
  const { discovery, watchers, nativeWatchers } = createDiscovery(async () => [uri], [folder]);
  const reasons = [];
  const subscriptions = discovery.watch((reason) => reasons.push(reason));
  await discovery.refresh(1);
  assert.equal(nativeWatchers.length, 0);
  assert.equal(watchers.length, 1);
  watchers[0].callbacks.change(uri);
  assert.deepEqual(reasons, ["project configuration changed"]);
  for (const subscription of subscriptions) subscription.dispose();
});
