export type PackageManager = "npm" | "pnpm" | "yarn" | "bun";

export interface CreateFrescoProjectOptions {
  directory: string;
  cwd?: string;
  packageManager?: PackageManager;
  force?: boolean;
}

export interface CreateFrescoProjectResult {
  targetDirectory: string;
  packageName: string;
  packageManager: PackageManager;
  files: readonly string[];
}
