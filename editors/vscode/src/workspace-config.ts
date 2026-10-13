import * as fs from "node:fs";
import * as path from "node:path";
import { workspace, type Disposable, type Uri } from "vscode";
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
    const watcher = workspace.createFileSystemWatcher(WORKSPACE_PROJECT_CONFIG_GLOB);
    const changed = (kind: string, uri: Uri) => {
      const accepted = !uri.path.split("/").some((part) => EXCLUDED_DIRECTORIES.includes(part));
      this.trace(`watch callback kind=${kind} accepted=${accepted} uri=${uri.toString()}`);
      if (accepted) {
        onChange("project configuration changed");
      }
    };
    return [
      watcher,
      watcher.onDidCreate((uri) => changed("create", uri)),
      watcher.onDidChange((uri) => changed("change", uri)),
      watcher.onDidDelete((uri) => changed("delete", uri)),
      workspace.onDidChangeWorkspaceFolders(() => {
        this.trace("workspace folders callback");
        onChange("workspace changed");
      }),
    ];
  }
}
