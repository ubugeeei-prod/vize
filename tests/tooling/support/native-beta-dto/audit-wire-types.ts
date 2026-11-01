export type Registration = {
  id: string;
  method: string;
  registerOptions: { watchers: Array<{ kind: number; globPattern: string }> };
};
export type Packet = {
  jsonrpc: string;
  id: string | number;
  method?: string;
  params: {
    registrations?: Registration[];
    file?: string;
    files?: string[];
    [name: string]: unknown;
  };
  result: { data?: string } | null;
};
export type Syscall = {
  at: string;
  op: string;
  fd: number;
  tid: number;
  data: Buffer;
  file: string;
  line: number;
};
export type Stream = {
  file: string;
  bytes: number;
  sha256: string;
  precedingNonProtocolBytes: number;
  packets: Packet[];
};
export type StreamPair = { fd: number; client?: Stream; server?: Stream };
export type Provider = { version: string; executable: string; executableSha256: string };
export type Source = {
  source: string;
  root: string;
  role: string;
  sdkVersion: string;
  sdkSha256: string;
  rows: Array<{ phase: string; encodedSourceSha256: string; lower: { refused: string } }>;
  observerDependencies: Array<{ path: string; sha256: string }>;
  observerSha256: string;
  attached: { root: string; rows: unknown[]; [field: string]: unknown };
};
export type Receipt = {
  scope: string;
  nativeOwner: number;
  nativeExit: number | undefined;
  nativeStdinEof: boolean;
  exitNotifications: number;
  apiAttached: boolean;
  registrationRefusals: number;
  stderr: { file: string; bytes: number; sha256: string };
  streams: StreamPair[];
  compilerKindPackets: Array<{ request: Packet; response: Packet | undefined }>;
};
