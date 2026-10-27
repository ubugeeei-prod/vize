/**
 * Package-manager table for the fresh-project release smoke (#3956).
 *
 * A matrix cell is a (package manager, project shape) pair and both are data.
 * This half holds the argv each manager needs; `smoke-release-init-shapes.ts`
 * holds the project shapes. Adding a manager is a new entry here rather than new
 * code in the driver.
 *
 * `bootstrap` materialises the project's own dependencies before `init` runs, so
 * `init` sees the lockfile a real project already has -- that lockfile is what
 * `detectPackageManager` reads, and it is what decides the installer the plan
 * prints. `redirect` forces every *transitive* resolution of a packed package
 * onto its tarball; direct dependencies are redirected on the command line
 * instead, because npm rejects an override that collides with a direct spec.
 *
 * `installFlags` deliberately omits `--legacy-peer-deps`, which the umbrella
 * install in `smoke-release-install.ts` needs: a real user installing the plan
 * does not pass it, so a genuine peer conflict in the published packages has to
 * red-light here.
 */

/** A package.json under construction; `redirect` adds its override table. */
export interface ProjectManifest {
  [field: string]: unknown;
  overrides?: Readonly<Record<string, string>>;
  resolutions?: Readonly<Record<string, string>>;
}

export type PackageManagerId = "npm" | "pnpm" | "yarn" | "bun" | "vp";

export interface PackageManager {
  readonly id: PackageManagerId;
  readonly binary: string;
  readonly binaryEnv: string;
  readonly corepackSpec?: string;
  readonly detectedPackageManager: string;
  readonly corsaInstallCommand: string;
  readonly lockfile: string;
  readonly bootstrapArgs: readonly string[];
  readonly installArgs: readonly string[];
  readonly installFlags: readonly string[];
  readonly environment: Readonly<Record<string, string>>;
  readonly projectFiles: Readonly<Record<string, string>>;
  readonly redirectPlannedDependencies: boolean;
  readonly runScriptArgs: (script: string, extra: readonly string[]) => string[];
  /** Returns extra project files when the manager reads redirects from one. */
  readonly redirect: (
    manifest: ProjectManifest,
    redirects: Readonly<Record<string, string>>,
  ) => Readonly<Record<string, string>> | undefined;
}

function pnpmWorkspaceOverrides(redirects: Readonly<Record<string, string>>): string {
  return [
    "overrides:",
    ...Object.entries(redirects).map(
      ([name, spec]) => `  ${JSON.stringify(name)}: ${JSON.stringify(spec)}`,
    ),
    "",
  ].join("\n");
}

export const PACKAGE_MANAGERS: Readonly<Record<PackageManagerId, PackageManager>> = {
  npm: {
    id: "npm",
    binary: "npm",
    binaryEnv: "NPM_BIN",
    detectedPackageManager: "npm",
    corsaInstallCommand: "npm",
    lockfile: "package-lock.json",
    bootstrapArgs: ["install", "--no-audit", "--fund=false", "--include=optional"],
    installArgs: ["install", "-D"],
    installFlags: ["--no-audit", "--fund=false", "--include=optional"],
    environment: {},
    projectFiles: {},
    redirectPlannedDependencies: false,
    runScriptArgs: (script, extra) =>
      extra.length === 0
        ? ["run", "--silent", script]
        : ["run", "--silent", script, "--", ...extra],
    redirect: (manifest, redirects) => {
      manifest.overrides = { ...manifest.overrides, ...redirects };
    },
  },
  pnpm: {
    id: "pnpm",
    binary: "pnpm",
    binaryEnv: "PNPM_BIN",
    corepackSpec: "pnpm@11.21.0",
    detectedPackageManager: "pnpm",
    corsaInstallCommand: "pnpm",
    lockfile: "pnpm-lock.yaml",
    bootstrapArgs: ["install", "--config.optional=true"],
    installArgs: ["add", "-D"],
    installFlags: ["--config.optional=true"],
    environment: {},
    projectFiles: {},
    redirectPlannedDependencies: true,
    // pnpm can reinstall before running a script. --silent hides that
    // installer's errors too, leaving CI with only an unexplained exit 1.
    runScriptArgs: (script, extra) => ["run", script, ...extra],
    redirect: (_manifest, redirects) => ({
      "pnpm-workspace.yaml": pnpmWorkspaceOverrides(redirects),
    }),
  },
  yarn: {
    id: "yarn",
    binary: "yarn",
    binaryEnv: "YARN_BIN",
    corepackSpec: "yarn@4.9.2",
    detectedPackageManager: "yarn",
    corsaInstallCommand: "yarn",
    lockfile: "yarn.lock",
    bootstrapArgs: ["install"],
    installArgs: ["add", "-D"],
    installFlags: [],
    environment: { YARN_ENABLE_IMMUTABLE_INSTALLS: "false" },
    // Yarn 4 defaults to Plug'n'Play. The CLI currently ships a node_modules
    // binary contract, so the release smoke pins the linker users need today.
    projectFiles: { ".yarnrc.yml": "nodeLinker: node-modules\n" },
    redirectPlannedDependencies: true,
    runScriptArgs: (script, extra) => ["run", "--silent", script, ...extra],
    redirect: (manifest, redirects) => {
      manifest.resolutions = { ...manifest.resolutions, ...redirects };
    },
  },
  bun: {
    id: "bun",
    binary: "bun",
    binaryEnv: "BUN_BIN",
    detectedPackageManager: "bun",
    corsaInstallCommand: "bun",
    lockfile: "bun.lock",
    bootstrapArgs: ["install"],
    installArgs: ["add", "-D"],
    installFlags: [],
    environment: {},
    projectFiles: {},
    redirectPlannedDependencies: true,
    runScriptArgs: (script, extra) => ["run", "--silent", script, ...extra],
    redirect: (manifest, redirects) => {
      manifest.overrides = { ...manifest.overrides, ...redirects };
    },
  },
  vp: {
    id: "vp",
    binary: "vp",
    binaryEnv: "VP_BIN",
    detectedPackageManager: "pnpm",
    corsaInstallCommand: "pnpm",
    lockfile: "pnpm-lock.yaml",
    bootstrapArgs: ["install"],
    installArgs: ["add", "-D"],
    installFlags: [],
    environment: {},
    projectFiles: {},
    redirectPlannedDependencies: true,
    runScriptArgs: (script, extra) =>
      extra.length === 0 ? ["run", script] : ["exec", "vize", "check", ...extra],
    redirect: (_manifest, redirects) => ({
      "pnpm-workspace.yaml": pnpmWorkspaceOverrides(redirects),
    }),
  },
};
