import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { stripTypeScriptTypes } from "node:module";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const source = fs.readFileSync(path.join(root, "editors/vscode/src/extension.ts"), "utf8");
const body = stripTypeScriptTypes(source, { mode: "transform" })
  .replace(/^import[\s\S]*?;\n/gm, "")
  .replaceAll("export async function", "async function");

function deferred() {
  let resolve;
  const promise = new Promise((complete) => {
    resolve = complete;
  });
  return { promise, resolve };
}

function createSync(refresh, activeClient, refuseLogging = false) {
  const operations = [];
  const logs = [];
  const timers = new Map();
  const started = deferred();
  let nextTimer = 0;
  const output = {
    appendLine(message) {
      if (refuseLogging && message.startsWith("[config ")) throw new Error("logger disposed");
      logs.push(message);
    },
  };
  class Discovery {
    hasConfig = true;
    refresh = refresh;
  }
  const context = vm.createContext({
    process: { env: { VIZE_TEST_ENABLE_HOST_COMMANDS: "1" } },
    Date,
    promisify: () => () => {},
    execFile: () => {},
    WorkspaceLspConfigDiscovery: Discovery,
    WORKSPACE_LSP_CONFIG_FILES: [],
    workspace: {
      getConfiguration: (name) => {
        operations.push(["configuration", name]);
        return { enabled: true };
      },
    },
    shouldStartFromConfiguration: (configuration) => configuration.enabled,
    setTimeout(callback, milliseconds) {
      const id = ++nextTimer;
      operations.push(["schedule", id, milliseconds]);
      timers.set(id, callback);
      return id;
    },
    clearTimeout(id) {
      operations.push(["cancel", id]);
      timers.delete(id);
    },
    output,
    activeClient,
    operations,
    started,
  });
  const control = new vm.Script(`${body}\n
    outputChannel = output;
    client = activeClient;
    updateStatusBar = (...args) => operations.push(["status", ...args]);
    startClient = async (_context, _config, generation) => {
      operations.push(["start", generation]); started.resolve();
    };
    ({ scheduleClientSync, syncClientToConfiguration, deactivate,
       setClient: (value) => { client = value; },
       state: () => ({ generation: configurationSyncGeneration, hasClient: Boolean(client) }) });
  `).runInContext(context);
  return { control, operations, logs, timers, started };
}

await test("actual sync retains one150ms timer, cancellation and stop-before-start with trace", async () => {
  const stopped = [];
  const current = createSync(async () => {}, {
    stop: async () => {
      stopped.push("stop");
    },
  });
  current.control.scheduleClientSync({}, "first callback");
  current.control.scheduleClientSync({}, "second callback");
  assert.deepEqual(current.operations, [
    ["schedule", 1, 150],
    ["cancel", 1],
    ["schedule", 2, 150],
  ]);
  assert.equal(current.timers.size, 1);
  current.timers.get(2)();
  await current.started.promise;
  assert.deepEqual(stopped, ["stop"]);
  assert.deepEqual(
    current.operations.filter(([kind]) => kind === "start").map((row) => [...row]),
    [["start", 1]],
  );
  assert.ok(current.logs.some((line) => line.includes("timer cancel generation=0")));
  assert.ok(
    current.logs.some((line) => line.includes("timer fire reason=second callback generation=0")),
  );
  assert.ok(current.logs.some((line) => line.includes("stop finish generation=1")));
});

await test("actual later refresh displaces earlier completion and preserves its original generation refusal", async () => {
  const first = deferred();
  const second = deferred();
  let calls = 0;
  const current = createSync(() => (++calls === 1 ? first.promise : second.promise));
  const oldSync = current.control.syncClientToConfiguration({}, "old callback");
  const newSync = current.control.syncClientToConfiguration({}, "new callback");
  second.resolve();
  await newSync;
  first.resolve();
  await oldSync;
  assert.deepEqual(
    current.operations.map((row) => [...row]),
    [
      ["configuration", "vize"],
      ["start", 2],
    ],
  );
  assert.ok(current.logs.some((line) => line.includes("sync refused generation=1 stage=refresh")));
  assert.equal(current.control.state().generation, 2);
});

await test("actual deactivate during stop refuses stale restart and cancels outstanding timer", async () => {
  const entered = deferred();
  const stopped = deferred();
  const current = createSync(async () => {}, {
    stop: () => {
      entered.resolve();
      return stopped.promise;
    },
  });
  const syncing = current.control.syncClientToConfiguration({}, "restart callback");
  await entered.promise;
  current.control.scheduleClientSync({}, "pending callback");
  await current.control.deactivate();
  stopped.resolve();
  await syncing;
  assert.equal(current.timers.size, 0);
  assert.equal(
    current.operations.some(([kind]) => kind === "start"),
    false,
  );
  assert.ok(
    current.logs.some((line) => line.includes("sync refused generation=1 stage=restart-stop")),
  );
  assert.equal(current.control.state().hasClient, false);
});

await test("actual refresh error remains identical even if diagnostic output refuses writes", async () => {
  const originalError = new Error("original findFiles rejection");
  const current = createSync(
    async () => {
      throw originalError;
    },
    undefined,
    true,
  );
  await assert.rejects(
    current.control.syncClientToConfiguration({}, "callback"),
    (error) => error === originalError,
  );
  assert.deepEqual(current.operations, []);
  assert.equal(current.control.state().generation, 1);
});

await test("project-file discovery retains the live client and pending settings restart", async () => {
  let refreshes = 0;
  let stops = 0;
  const current = createSync(
    async () => {
      refreshes++;
    },
    {
      stop: async () => {
        stops++;
      },
    },
  );
  current.control.scheduleClientSync({}, "configuration changed");
  current.control.scheduleClientSync({}, "project configuration changed");
  await Promise.resolve();
  assert.equal(refreshes, 1);
  assert.equal(stops, 0);
  assert.equal(current.control.state().generation, 0);
  assert.equal(current.control.state().hasClient, true);
  assert.deepEqual(current.operations, [["schedule", 1, 150]]);
  assert.equal(current.timers.size, 1);
  current.timers.get(1)();
  await current.started.promise;
  assert.equal(refreshes, 2);
  assert.equal(stops, 1);
  assert.deepEqual(
    current.operations.filter(([kind]) => kind === "start").map((row) => [...row]),
    [["start", 1]],
  );
});

await test("a queued project discovery event retains a client started before its timer fires", async () => {
  let refreshes = 0;
  let stops = 0;
  const current = createSync(async () => {
    refreshes++;
  });
  current.control.scheduleClientSync({}, "project configuration changed");
  current.control.setClient({
    stop: async () => {
      stops++;
    },
  });
  current.timers.get(1)();
  await Promise.resolve();
  assert.equal(refreshes, 1);
  assert.equal(stops, 0);
  assert.deepEqual(current.operations, [["schedule", 1, 150]]);
  assert.equal(current.control.state().generation, 0);
  assert.equal(current.control.state().hasClient, true);
});

await test("first project configuration still starts a client through debounced discovery", async () => {
  let refreshes = 0;
  const current = createSync(async () => {
    refreshes++;
  });
  current.control.scheduleClientSync({}, "project configuration changed");
  assert.deepEqual(current.operations, [["schedule", 1, 150]]);
  current.timers.get(1)();
  await current.started.promise;
  assert.equal(refreshes, 1);
  assert.deepEqual(
    current.operations.filter(([kind]) => kind === "start").map((row) => [...row]),
    [["start", 1]],
  );
});
