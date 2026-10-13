import * as path from "path";

/**
 * Server-path decisions that depend on Workspace Trust.
 *
 * A cloned repository controls its workspace settings and its files, so in an
 * untrusted workspace neither may select the executable the extension runs.
 * The package manifest declares limited untrusted-workspace support; these
 * functions are the second layer. (#969)
 */

/** The scopes of the `vize.serverPath` setting, as `inspect` reports them. */
export interface ServerPathSetting {
  readonly globalValue?: string;
  readonly workspaceValue?: string;
  readonly workspaceFolderValue?: string;
}

/**
 * The configured server path to use. User and machine settings always count;
 * workspace and folder settings count only in a trusted workspace.
 */
export function trustedServerPath(
  setting: ServerPathSetting | undefined,
  isTrusted: boolean,
): string | undefined {
  if (!setting) {
    return undefined;
  }
  const candidates = [
    setting.globalValue,
    isTrusted ? setting.workspaceValue : undefined,
    isTrusted ? setting.workspaceFolderValue : undefined,
  ];
  for (const raw of candidates) {
    const trimmed = raw?.trim();
    if (trimmed) {
      return trimmed;
    }
  }
  return undefined;
}

/**
 * Development builds inside the workspace folders (`target/{release,debug}`).
 * An untrusted workspace contributes none: a cloned repository could ship a
 * binary at exactly those paths.
 */
export function workspaceDevServerPaths(
  folders: readonly string[],
  exeName: string,
  isTrusted: boolean,
): string[] {
  if (!isTrusted) {
    return [];
  }
  return folders.flatMap((folder) => [
    path.join(folder, "target", "release", exeName),
    path.join(folder, "target", "debug", exeName),
  ]);
}
