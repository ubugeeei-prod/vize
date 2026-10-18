import fs from "node:fs";
import path from "node:path";
import { stripTypeScriptTypes } from "node:module";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { EventEmitter } from "node:events";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const source = fs.readFileSync(path.join(root, "editors/vscode/src/workspace-config.ts"), "utf8");

export const CONFIG_GLOB = "original-project-config-glob";

export function fileUri(pathname, scheme = "file") {
  return { scheme, path: pathname, fsPath: pathname, toString: () => `${scheme}://${pathname}` };
}

function workspaceFolder(pathname, index = 0) {
  const uri = typeof pathname === "string" ? fileUri(pathname) : pathname;
  return { uri, name: path.basename(uri.path), index };
}

export function createDiscovery(
  findFiles = async () => [],
  initialFolders = ["/workspace"],
  { sourceText = source, nativeWatchError } = {},
) {
  const body = stripTypeScriptTypes(sourceText, { mode: "transform" })
    .replace(/^import[\s\S]*?;\n/gm, "")
    .replace("export class WorkspaceLspConfigDiscovery", "class WorkspaceLspConfigDiscovery");

  const operations = [];
  const trace = [];
  const callbacks = {};
  const watchers = [];
  const nativeWatchers = [];
  const nativeFs = {
    ...fs,
    watch: (parent, options, callback) => {
      operations.push(["nativeWatch", parent, options]);
      if (nativeWatchError) throw nativeWatchError;
      const watcher = new EventEmitter();
      const index = nativeWatchers.length;
      Object.assign(watcher, { parent, options, callback, closed: false });
      watcher.close = () => {
        if (watcher.closed) return;
        watcher.closed = true;
        operations.push(["nativeClose", index]);
        watcher.emit("close");
      };
      nativeWatchers.push(watcher);
      return watcher;
    },
  };
  const disposable = (name) => ({ dispose: () => operations.push(["dispose", name]) });
  const workspace = {
    workspaceFolders: initialFolders.map(workspaceFolder),
    getWorkspaceFolder: (uri) =>
      workspace.workspaceFolders
        .filter(
          ({ uri: folder }) =>
            uri.scheme === folder.scheme &&
            (uri.path === folder.path || uri.path.startsWith(`${folder.path}/`)),
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
    fs: nativeFs,
    path,
    workspace,
    Uri: { file: fileUri, joinPath: (uri, part) => fileUri(path.posix.join(uri.path, part)) },
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
  return { discovery, operations, callbacks, trace, watchers, nativeWatchers, setFolders };
}
