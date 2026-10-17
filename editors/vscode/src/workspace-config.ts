import * as fs from "node:fs";
import * as path from "node:path";
import {
  RelativePattern,
  workspace,
  type Disposable,
  type FileSystemWatcher,
  type Uri,
} from "vscode";
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

  constructor(
    private readonly projectConfigFiles: readonly string[],
    private readonly trace: (message: string) => void = () => {},
  ) {}

  get hasConfig(): boolean {
    return (
      this.discovered ||
      (workspace.workspaceFolders ?? []).some((folder) =>
        [...this.projectConfigFiles, ...WORKSPACE_PROJECT_CONFIG_FILES].some((filename) =>
          fs.existsSync(path.join(folder.uri.fsPath, filename)),
        ),
      )
    );
  }

  async refresh(generation?: number): Promise<void> {
    const refreshGeneration = ++this.refreshGeneration;
    this.trace(`findFiles enter generation=${generation}`);
    try {
      const discovered =
        (
          await workspace.findFiles(
            WORKSPACE_PROJECT_CONFIG_GLOB,
            `**/{${EXCLUDED_DIRECTORIES.join(",")}}/**`,
            1,
          )
        ).length > 0;
      if (refreshGeneration !== this.refreshGeneration) {
        this.trace(`findFiles refused generation=${generation} stage=superseded`);
        return;
      }
      this.discovered = discovered;
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
        const watcher = workspace.createFileSystemWatcher(
          new RelativePattern(folder, WORKSPACE_PROJECT_CONFIG_GLOB),
        );
        const changed = (kind: string, uri: Uri) => {
          if (
            disposed ||
            leases.get(key)?.watcher !== watcher ||
            workspace.getWorkspaceFolder(uri)?.uri.toString() !== key
          ) {
            return;
          }
          const accepted = !uri.path.split("/").some((part) => EXCLUDED_DIRECTORIES.includes(part));
          this.trace(`watch callback kind=${kind} accepted=${accepted} uri=${uri.toString()}`);
          if (accepted) onChange("project configuration changed");
        };
        leases.set(key, {
          watcher,
          subscriptions: [
            watcher,
            watcher.onDidCreate((uri) => changed("create", uri)),
            watcher.onDidChange((uri) => changed("change", uri)),
            watcher.onDidDelete((uri) => changed("delete", uri)),
          ],
        });
      }
    };
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
          for (const lease of leases.values()) {
            for (const subscription of lease.subscriptions) subscription.dispose();
          }
          leases.clear();
        },
      },
    ];
  }
}
