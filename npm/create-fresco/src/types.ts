/** Package managers whose commands can be emitted by create-fresco. */
export type PackageManager = "npm" | "pnpm" | "yarn" | "bun";

/** Options for materializing a Fresco project without installing dependencies. */
export interface CreateFrescoProjectOptions {
  /** Target directory, resolved from `cwd`. */
  directory: string;
  /** Base directory used to resolve `directory`. @default process.cwd() */
  cwd?: string;
  /**
   * Package manager written into generated instructions.
   * @default auto-detected from npm_config_user_agent, otherwise "pnpm"
   */
  packageManager?: PackageManager;
  /** Allow replacing scaffold-owned files in a non-empty target. @default false */
  force?: boolean;
}

/** Deterministic description of a materialized Fresco project. */
export interface CreateFrescoProjectResult {
  /** Absolute directory containing the generated project. */
  targetDirectory: string;
  /** Normalized npm package name written to `package.json`. */
  packageName: string;
  /** Package manager selected for generated instructions. */
  packageManager: PackageManager;
  /** Relative paths of every scaffold-owned file written by the operation. */
  files: readonly string[];
}
