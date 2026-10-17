import type { OutputChannel, QuickPickItem, StatusBarItem } from "vscode";
import type { LspInitializationOptions } from "./extension-core.js";

export type ServerCandidateSource =
  | "configured"
  | "bundled"
  | "development"
  | "release"
  | "cargo"
  | "path";
export type ServerCandidate = {
  path: string;
  source: ServerCandidateSource;
};
export type InspectedServerCandidate = ServerCandidate & {
  version?: string;
  versionError?: string;
};
export type ReleaseServerTarget = {
  archiveName: string;
  targetName: string;
};
export type VizeStatus = "disabled" | "starting" | "ready" | "missing-server" | "failed";
export type VizeStatusAction =
  | "recommended"
  | "lintOnly"
  | "selectServer"
  | "restart"
  | "settings"
  | "output"
  | "disable";
export type VizeStatusQuickPickItem = QuickPickItem & {
  action: VizeStatusAction;
};

/** The lifecycle owns these fields; setup and discovery observe the same live values. */
export interface ExtensionRuntimeState {
  readonly outputChannel: OutputChannel;
  readonly statusBarItem: StatusBarItem | undefined;
  selectedServerCandidate: InspectedServerCandidate | undefined;
  readonly activeInitializationOptions: LspInitializationOptions;
  currentStatus: VizeStatus;
  currentStatusDetail: string;
  readonly configurationSyncGeneration: number;
  suppressConfigurationSync: boolean;
}
