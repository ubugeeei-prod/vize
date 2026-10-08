export type Packet = Record<string, unknown>;
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
  keepStdinOpenAfterExit?: boolean;
  exitTimeoutMs?: number;
};
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

export function validateInstalledLaunch(launch: InstalledAliasLaunch): void {
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
