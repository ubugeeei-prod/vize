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

export class WorkspaceLspConfigDiscovery {
  private discovered = false;
  private refreshGeneration = 0;
  private knownConfigurations: readonly Uri[] = [];
  private readonly configurationWatchers = new Set<() => void>();

  constructor(
    private readonly dedicatedConfigFiles: readonly string[],
    private readonly trace: (message: string) => void = () => {},
  ) {}

  get hasConfig(): boolean {
    return (
      this.discovered ||
      (workspace.workspaceFolders ?? []).some((folder) =>
        [...this.dedicatedConfigFiles, ...WORKSPACE_PROJECT_CONFIG_FILES].some((filename) =>
          fs.existsSync(path.join(folder.uri.fsPath, filename)),
        ),
      )
    );
  }

  async refresh(generation?: number): Promise<void> {
    const refreshGeneration = ++this.refreshGeneration;
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
      this.trace(`findFiles return generation=${generation} discovered=${this.discovered}`);
    } catch (error) {
      this.trace(`findFiles error generation=${generation}`);
      throw error;
    }
  }

  describeCapabilities(options: LspInitializationOptions): string {
    return Object.keys(options).length === 0 && this.hasConfig
      ? "workspace configuration and defaults"
      : describeCapabilities(options);
  }

  watch(onChange: (reason: string) => void): Disposable[] {
    let disposed = false;
    const leases = new Map<string, { watcher: FileSystemWatcher; subscriptions: Disposable[] }>();
    const parents = new Map<
      string,
      { owner: string; paths: Set<string>; subscriptions: Disposable[] }
    >();
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
      const current = new Map<string, { uri: Uri; owner: string; paths: Set<string> }>();
      for (const uri of this.knownConfigurations) {
        const owner = workspace.getWorkspaceFolder(uri)?.uri.toString();
        if (!owner || uri.path.split("/").some((part) => EXCLUDED_DIRECTORIES.includes(part))) {
          continue;
        }
        const parent = Uri.joinPath(uri, "..");
        const key = parent.toString();
        const group = current.get(key) ?? { uri: parent, owner, paths: new Set<string>() };
        group.paths.add(uri.toString());
        current.set(key, group);
      }
      for (const [key, lease] of parents) {
        if (current.get(key)?.owner !== lease.owner) {
          parents.delete(key);
          for (const subscription of lease.subscriptions) subscription.dispose();
        }
      }
      for (const [key, group] of current) {
        const previous = parents.get(key);
        if (previous) {
          previous.paths = group.paths;
          continue;
        }
        // Existing files must not depend on a recursive watcher discovering a
        // newly created ancestor. Request their concrete parent before startup.
        this.trace(`watch register parent=${key} workspace=${group.owner}`);
        const watcher = workspace.createFileSystemWatcher(new RelativePattern(group.uri, "*"));
        const lease = { owner: group.owner, paths: group.paths, subscriptions: [] as Disposable[] };
        const notify = (kind: string, uri: Uri) => {
          if (lease.paths.has(uri.toString())) {
            changed(kind, uri, lease.owner, parents.get(key) === lease);
          }
        };
        parents.set(key, lease);
        lease.subscriptions = [
          watcher,
          watcher.onDidCreate((uri) => notify("create", uri)),
          watcher.onDidChange((uri) => notify("change", uri)),
          watcher.onDidDelete((uri) => notify("delete", uri)),
        ];
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
          if (parent?.owner === key && parent.paths.has(uri.toString())) return;
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
          for (const lease of parents.values()) {
            for (const subscription of lease.subscriptions) subscription.dispose();
          }
          parents.clear();
        },
      },
    ];
  }
}
