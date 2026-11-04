export type JwtPayload = Readonly<Record<string, unknown>>;

export interface PermissionBinding {
  scope: string;
  packageName: string;
  version: string;
  tarballHash: string;
}

export interface ProvenanceRoute {
  scope: string | null;
  packageName: string | null;
  version: string | null;
}

export interface ProvenanceReceipt {
  attempted: boolean;
  accepted: boolean;
  reason: string | null;
  registryStatus: number | null;
  binding: PermissionBinding | null;
}

export interface PublishOptions {
  command: readonly string[];
  cwd?: string;
  env?: NodeJS.ProcessEnv;
  upstream?: string;
  now?: () => number;
  signals?: boolean;
}

export interface PublishResult {
  code: number;
  receipt: ProvenanceReceipt;
}
