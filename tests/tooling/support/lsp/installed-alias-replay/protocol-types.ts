export type Packet = Record<string, unknown>;
export type InstalledAliasInitializationOptions = {
  editor?: boolean;
  lint?: boolean;
  typecheck?: boolean;
  formatting?: boolean;
  hover?: boolean;
  crossFile?: boolean;
};
export type InstalledAliasLaunch = {
  nodePath: string;
  cliPath: string;
  custodyHookPath: string;
  projectRoot: string;
  env: NodeJS.ProcessEnv;
  outputRoot: string;
  timeoutMs: number;
  crossFile: boolean;
  lint?: boolean;
  initializationOptions?: InstalledAliasInitializationOptions;
  keepStdinOpenAfterExit?: boolean;
  exitTimeoutMs?: number;
};

/** Absent retains the original initializer; explicit false values stay false. */
export function installedInitializationOptions(
  launch: Pick<InstalledAliasLaunch, "initializationOptions" | "lint" | "crossFile">,
): InstalledAliasInitializationOptions {
  const options = launch.initializationOptions;
  if (options === undefined)
    return {
      lint: launch.lint ?? false,
      typecheck: true,
      hover: true,
      crossFile: launch.crossFile,
    };
  if (options === null || typeof options !== "object" || Array.isArray(options))
    throw new Error("invalid initialization options");
  for (const [name, value] of Object.entries(options)) {
    if (!["editor", "lint", "typecheck", "formatting", "hover", "crossFile"].includes(name))
      throw new Error("unknown initialization option: " + name);
    if (value !== undefined && typeof value !== "boolean")
      throw new Error("initialization option must be boolean: " + name);
  }
  return { ...options };
}

/** The actual launch path and pure controls share the complete initializer. */
export function installedInitializeParams(
  launch: Pick<
    InstalledAliasLaunch,
    "projectRoot" | "initializationOptions" | "lint" | "crossFile"
  >,
) {
  return {
    processId: null,
    rootUri: pathToFileURL(realpathSync(launch.projectRoot)).href,
    capabilities: {},
    initializationOptions: installedInitializationOptions(launch),
  };
}
export type Outcome = {
  id: number;
  method: string;
  status: string;
  response?: Packet;
  error?: string;
  observed?: boolean;
};
export type Pending = {
  resolve: (packet: Packet) => void;
  reject: (error: Error) => void;
  timer: NodeJS.Timeout;
  outcome: Outcome;
  observed: boolean;
};
export type PublicationWait = {
  uri: string;
  version: number;
  after: number;
  packets: Packet[];
  count: number;
  resolve: (packets: Packet[]) => void;
  reject: (error: Error) => void;
  timer: NodeJS.Timeout;
};
export class InstalledAliasError extends Error {
  readonly packet: Packet | undefined;
  constructor(message: string, packet?: Packet) {
    super(message);
    this.name = "InstalledAliasError";
    this.packet = packet;
  }
}
import path from "node:path";
import { realpathSync } from "node:fs";
import { pathToFileURL } from "node:url";

export function validateInstalledLaunch(launch: InstalledAliasLaunch): void {
  installedInitializationOptions(launch);
  for (const name of [
    "nodePath",
    "cliPath",
    "custodyHookPath",
    "projectRoot",
    "outputRoot",
  ] as const) {
    if (!path.isAbsolute(launch[name])) throw new Error(`${name} must be absolute`);
  }
  if (!Number.isSafeInteger(launch.timeoutMs) || launch.timeoutMs <= 0)
    throw new Error("invalid deadline");
  if (
    launch.exitTimeoutMs !== undefined &&
    (!Number.isSafeInteger(launch.exitTimeoutMs) || launch.exitTimeoutMs <= 0)
  )
    throw new Error("invalid exit deadline");
}
