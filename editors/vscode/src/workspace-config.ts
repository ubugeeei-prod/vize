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

  constructor(private readonly dedicatedConfigFiles: readonly string[]) {}

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

  async refresh(): Promise<void> {
    this.discovered =
      (
        await workspace.findFiles(
          WORKSPACE_PROJECT_CONFIG_GLOB,
          `**/{${EXCLUDED_DIRECTORIES.join(",")}}/**`,
          1,
        )
      ).length > 0;
  }

  describeCapabilities(options: LspInitializationOptions): string {
    return Object.keys(options).length === 0 && this.hasConfig
      ? "workspace configuration and defaults"
      : describeCapabilities(options);
  }

  watch(onChange: (reason: string) => void): Disposable[] {
    const watcher = workspace.createFileSystemWatcher(WORKSPACE_PROJECT_CONFIG_GLOB);
    const changed = (uri: Uri) => {
      if (!uri.path.split("/").some((part) => EXCLUDED_DIRECTORIES.includes(part))) {
        onChange("project configuration changed");
      }
    };
    return [
      watcher,
      watcher.onDidCreate(changed),
      watcher.onDidDelete(changed),
      workspace.onDidChangeWorkspaceFolders(() => onChange("workspace changed")),
    ];
  }
}
