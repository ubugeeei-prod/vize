import * as fs from "node:fs";
import * as path from "node:path";
import { RelativePattern, Uri, workspace, type Disposable, type FileSystemWatcher } from "vscode";
import {
  WORKSPACE_PROJECT_CONFIG_FILES,
  WORKSPACE_PROJECT_CONFIG_GLOB,
  describeCapabilities,
  type LspInitializationOptions,
} from "./extension-core.js";

const EXCLUDED_DIRECTORIES = ["node_modules", ".git", "target", "dist", "build"];
/** Directories one startup decision may list before it waits for the complete search. */
const NEAR_DIRECTORY_BUDGET = 32;

type ParentWatch = {
  owner: string;
  paths: Set<string>;
  files: Map<string, Uri>;
  watcher?: fs.FSWatcher;
  active: boolean;
};

export class WorkspaceLspConfigDiscovery {
  private discovered = false;
  private nearMatch = false;
  private refreshGeneration = 0;
  private rootConfig: { generation: number; found: boolean } | undefined;
  private completedGeneration = -1;
  private knownConfigurations: readonly Uri[] = [];
  private readonly configurationWatchers = new Set<() => void>();

  constructor(
    private readonly projectConfigFiles: readonly string[],
    private readonly trace: (message: string) => void = () => {},
  ) {}

  get hasConfig(): boolean {
    return this.discovered || this.nearMatch || this.hasRootConfig;
  }

  /**
   * Lists a few directories within two levels of each folder. Any config found
   * there is one the workspace-wide search also returns, so until that search
   * completes it decides startup.
   */
  async findNearConfig(): Promise<boolean> {
    const refreshGeneration = this.refreshGeneration;
    for (const folder of workspace.workspaceFolders ?? []) {
      const hidden = basenameExcludes(
        workspace.getConfiguration("files", folder.uri).get("exclude"),
      );
      if (folder.uri.scheme !== "file" || !hidden || !(await nearConfig(folder.uri.fsPath, hidden)))
        continue;
      // A newer search owns the decision once this one is superseded.
      this.nearMatch =
        refreshGeneration === this.refreshGeneration &&
        refreshGeneration !== this.completedGeneration;
      return this.nearMatch;
    }
    return false;
  }

  /**
   * A workspace folder's own config; the workspace-wide search cannot change it.
   * Checked once per refresh: config and folder changes always start a refresh,
   * and on a WSL2 Windows mount every check is a 9P round trip.
   */
  get hasRootConfig(): boolean {
    if (this.rootConfig?.generation === this.refreshGeneration) return this.rootConfig.found;
    const filenames = new Set([...this.projectConfigFiles, ...WORKSPACE_PROJECT_CONFIG_FILES]);
    const found = (workspace.workspaceFolders ?? []).some((folder) =>
      [...filenames].some((filename) => fs.existsSync(path.join(folder.uri.fsPath, filename))),
    );
    this.rootConfig = { generation: this.refreshGeneration, found };
    return found;
  }

  async refresh(generation?: number): Promise<void> {
    const refreshGeneration = ++this.refreshGeneration;
    this.nearMatch = false;
    this.trace(`findFiles enter generation=${generation}`);
    try {
      const configurations = await workspace.findFiles(
        WORKSPACE_PROJECT_CONFIG_GLOB,
        `**/{${EXCLUDED_DIRECTORIES.join(",")}}/**`,
      );
      if (refreshGeneration !== this.refreshGeneration) {
        this.trace(`findFiles refused generation=${generation} stage=superseded`);
        return;
      }
      this.knownConfigurations = configurations;
      for (const reconcile of this.configurationWatchers) reconcile();
      this.discovered = configurations.length > 0;
      this.nearMatch = false;
      this.completedGeneration = refreshGeneration;
      this.trace(`findFiles return generation=${generation} discovered=${this.discovered}`);
    } catch (error) {
      this.trace(`findFiles error generation=${generation}`);
      throw error;
    }
  }

  describeCapabilities(options: LspInitializationOptions): string {
    return describeCapabilities(options, this.hasConfig);
  }

  watch(onChange: (reason: string) => void): Disposable[] {
    let disposed = false;
    const leases = new Map<string, { watcher: FileSystemWatcher; subscriptions: Disposable[] }>();
    const parents = new Map<string, ParentWatch>();
    const closeParent = (key: string, lease: ParentWatch) => {
      if (parents.get(key) !== lease) return;
      parents.delete(key);
      lease.active = false;
      lease.watcher?.close();
    };
    const changed = (kind: string, uri: Uri, key: string, active: boolean) => {
      const owner =
        disposed || !active ? undefined : workspace.getWorkspaceFolder(uri)?.uri.toString();
      this.trace(
        `watch event kind=${kind} uri=${uri.toString()} workspace=${key} disposed=${disposed} active=${active} owner=${owner}`,
      );
      if (disposed || !active || owner !== key) return;
      const accepted = !uri.path.split("/").some((part) => EXCLUDED_DIRECTORIES.includes(part));
      this.trace(`watch callback kind=${kind} accepted=${accepted} uri=${uri.toString()}`);
      if (accepted) onChange("project configuration changed");
    };
    const reconcileParents = () => {
      if (disposed) return;
      const current = new Map<
        string,
        { uri: Uri; owner: string; paths: Set<string>; files: Map<string, Uri> }
      >();
      for (const uri of this.knownConfigurations) {
        const owner = workspace.getWorkspaceFolder(uri)?.uri.toString();
        if (
          uri.scheme !== "file" ||
          !owner ||
          uri.path.split("/").some((part) => EXCLUDED_DIRECTORIES.includes(part))
        ) {
          continue;
        }
        const parent = Uri.joinPath(uri, "..");
        const key = parent.toString();
        const group = current.get(key) ?? {
          uri: parent,
          owner,
          paths: new Set<string>(),
          files: new Map<string, Uri>(),
        };
        group.paths.add(uri.toString());
        group.files.set(path.basename(uri.fsPath), uri);
        current.set(key, group);
      }
      for (const [key, lease] of parents) {
        if (current.get(key)?.owner !== lease.owner) {
          closeParent(key, lease);
        }
      }
      for (const [key, group] of current) {
        const previous = parents.get(key);
        if (previous) {
          previous.paths = group.paths;
          previous.files = group.files;
          continue;
        }
        // VS Code restricts nonrecursive workspace watches to watcherExclude
        // paths. A direct native handle also covers ordinary existing configs.
        this.trace(`watch register native parent=${key} workspace=${group.owner}`);
        const lease: ParentWatch = { ...group, active: false };
        parents.set(key, lease);
        try {
          lease.watcher = fs.watch(
            group.uri.fsPath,
            { recursive: false, persistent: false, encoding: "utf8" },
            (kind, filename) => {
              if (parents.get(key) !== lease || !lease.active) return;
              if (typeof filename !== "string" || filename.length === 0) {
                this.trace(`watch native filename unavailable parent=${key}`);
                closeParent(key, lease);
                return;
              }
              const uri = lease.files.get(filename);
              if (uri) changed(kind, uri, lease.owner, true);
            },
          );
          lease.active = true;
          lease.watcher.on("error", (error) => {
            if (parents.get(key) !== lease) return;
            this.trace(`watch native error parent=${key} error=${String(error)}`);
            closeParent(key, lease);
          });
          lease.watcher.on("close", () => {
            if (parents.get(key) !== lease) return;
            this.trace(`watch native closed parent=${key}`);
            closeParent(key, lease);
          });
        } catch (error) {
          this.trace(`watch native error parent=${key} error=${String(error)}`);
          closeParent(key, lease);
        }
      }
    };
    const reconcile = () => {
      const folders = workspace.workspaceFolders ?? [];
      const current = new Set(folders.map((folder) => folder.uri.toString()));
      for (const [key, lease] of leases) {
        if (!current.has(key)) {
          leases.delete(key);
          for (const subscription of lease.subscriptions) subscription.dispose();
        }
      }
      for (const folder of folders) {
        const key = folder.uri.toString();
        if (leases.has(key)) continue;
        // A string glob only subscribes to the shared workspace watcher's
        // events. Request this folder's recursive watch explicitly at startup.
        this.trace(`watch register workspace=${key} pattern=${WORKSPACE_PROJECT_CONFIG_GLOB}`);
        const watcher = workspace.createFileSystemWatcher(
          new RelativePattern(folder, WORKSPACE_PROJECT_CONFIG_GLOB),
        );
        const notify = (kind: string, uri: Uri) => {
          const parent = parents.get(Uri.joinPath(uri, "..").toString());
          if (parent?.active && parent.owner === key && parent.paths.has(uri.toString())) return;
          changed(kind, uri, key, leases.get(key)?.watcher === watcher);
        };
        leases.set(key, {
          watcher,
          subscriptions: [
            watcher,
            watcher.onDidCreate((uri) => notify("create", uri)),
            watcher.onDidChange((uri) => notify("change", uri)),
            watcher.onDidDelete((uri) => notify("delete", uri)),
          ],
        });
      }
      reconcileParents();
    };
    this.configurationWatchers.add(reconcileParents);
    reconcile();
    return [
      workspace.onDidChangeWorkspaceFolders(() => {
        if (disposed) return;
        reconcile();
        this.trace("workspace folders callback");
        onChange("workspace changed");
      }),
      {
        dispose: () => {
          disposed = true;
          this.configurationWatchers.delete(reconcileParents);
          for (const lease of leases.values()) {
            for (const subscription of lease.subscriptions) subscription.dispose();
          }
          leases.clear();
          for (const [key, lease] of parents) closeParent(key, lease);
        },
      },
    ];
  }
}

const isProjectConfigName = (name: string) =>
  (WORKSPACE_PROJECT_CONFIG_FILES as readonly string[]).includes(name) ||
  /^tsconfig\..*\.json$/.test(name);

/** `files.exclude` also hides results of the complete search; only `**\/name` keys are mirrored. */
function basenameExcludes(setting: unknown): Set<string> | undefined {
  const names = new Set(EXCLUDED_DIRECTORIES.map((name) => name.toLowerCase()));
  if (setting === undefined || setting === null) return names;
  if (typeof setting !== "object") return undefined;
  for (const [pattern, enabled] of Object.entries(setting)) {
    if (enabled === false) continue;
    const name = /^\*\*\/([^*?[\]{}/\\!]+)$/.exec(pattern)?.[1];
    if (enabled !== true || !name) return undefined;
    names.add(name.toLowerCase());
  }
  return names;
}

/** Breadth-first over the folder and two directory levels, skipping links and dot directories. */
async function nearConfig(root: string, hidden: ReadonlySet<string>): Promise<boolean> {
  let level = [root];
  let budget = NEAR_DIRECTORY_BUDGET;
  for (let depth = 0; depth <= 2 && level.length > 0; depth++) {
    const next: string[] = [];
    for (const directory of level) {
      if (budget-- <= 0) return false;
      let entries: fs.Dirent[];
      try {
        entries = await fs.promises.readdir(directory, { withFileTypes: true });
      } catch {
        continue;
      }
      for (const entry of entries) {
        // Case-folded: a glob that matches case-insensitively hides no less.
        if (hidden.has(entry.name.toLowerCase())) continue;
        if (entry.isFile() && isProjectConfigName(entry.name)) return true;
        if (entry.isDirectory() && !entry.name.startsWith(".")) {
          next.push(path.join(directory, entry.name));
        }
      }
    }
    level = next;
  }
  return false;
}
