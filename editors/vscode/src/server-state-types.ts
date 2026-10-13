import type { QuickPickItem } from "vscode";

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
