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
};
export type Outcome = {
  id: number;
  method: string;
  status: string;
  response?: Packet;
  error?: string;
};
export type Pending = {
  resolve: (packet: Packet) => void;
  reject: (error: Error) => void;
  timer: NodeJS.Timeout;
  outcome: Outcome;
};
export type PublicationWait = {
  uri: string;
  version: number;
  after: number;
  resolve: (packet: Packet) => void;
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
