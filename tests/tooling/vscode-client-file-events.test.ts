import assert from "node:assert/strict";
import fs from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const source = fs.readFileSync(path.join(root, "editors/vscode/src/client-options.ts"), "utf8");

interface Watcher {
  glob: string;
  disposed: boolean;
  dispose(): void;
}

interface ClientOptionsModule {
  createClientOptions(...args: unknown[]): { synchronize: { fileEvents: unknown } };
  disposeClientFileEvents(): void;
}

await test("restarted clients share one pair of file watchers until deactivation", () => {
  const created: Watcher[] = [];
  const context = vm.createContext({
    workspace: {
      createFileSystemWatcher: (glob: string) => {
        const watcher: Watcher = {
          glob,
          disposed: false,
          dispose: () => {
            watcher.disposed = true;
          },
        };
        created.push(watcher);
        return watcher;
      },
    },
    createDocumentSelector: () => [],
    createAutoInsertMiddleware: () => ({}),
    withJsxRouting: (_client: unknown, middleware: unknown) => middleware,
  });
  const body = stripTypeScriptTypes(source, { mode: "transform" })
    .replace(/^import[\s\S]*?;\n/gm, "")
    .replace(/^export /gm, "");
  const control: ClientOptionsModule = new vm.Script(
    `${body}\n({ createClientOptions, disposeClientFileEvents });`,
  ).runInContext(context);
  const options = Array.from({ length: 10 }, () =>
    control.createClientOptions({}, () => undefined, {}, {}),
  );
  assert.equal(created.length, 2, "ten client starts create one pair of watchers");
  assert.deepEqual(
    created.map((watcher) => watcher.glob),
    ["**/*.vue", "**/*.{html,htm}"],
  );
  for (const option of options) {
    assert.equal(option.synchronize.fileEvents, options[0]?.synchronize.fileEvents);
  }
  control.disposeClientFileEvents();
  assert.deepEqual(
    created.map((watcher) => watcher.disposed),
    [true, true],
  );
  control.createClientOptions({}, () => undefined, {}, {});
  assert.equal(created.length, 4, "a reactivated extension creates a fresh pair");
});
