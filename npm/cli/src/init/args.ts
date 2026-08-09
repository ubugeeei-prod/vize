import { FEATURE_IDS, type FeatureId } from "./select.js";

export type BundlerOverride = "vite" | "nuxt" | null;

export interface InitArgs {
  readonly root: string | null;
  /** Explicit per-feature choices. Absent entries fall back to detection. */
  readonly overrides: Readonly<Partial<Record<FeatureId, boolean>>>;
  readonly bundlerOverride: BundlerOverride;
  readonly yes: boolean;
  readonly dryRun: boolean;
  readonly install: boolean;
  readonly packageManager: string | null;
  readonly help: boolean;
}

const PACKAGE_MANAGERS = ["pnpm", "npm", "yarn", "bun", "vp"] as const;

/**
 * Parses `vize init` arguments.
 *
 * `--yes` is the only switch that disables prompting. Per-feature flags without
 * it still prompt, using the flags as the pre-ticked defaults, which keeps a
 * half-typed command from silently writing files.
 */
export function parseInitArgs(args: readonly string[]): InitArgs {
  const overrides: Partial<Record<FeatureId, boolean>> = {};
  let root: string | null = null;
  let bundlerOverride: BundlerOverride = null;
  let yes = false;
  let dryRun = false;
  let install = true;
  let packageManager: string | null = null;
  let help = false;

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index]!;
    if (arg === "-h" || arg === "--help") {
      help = true;
      continue;
    }
    if (arg === "-y" || arg === "--yes") {
      yes = true;
      continue;
    }
    if (arg === "--dry-run") {
      dryRun = true;
      continue;
    }
    if (arg === "--no-install") {
      install = false;
      continue;
    }
    if (arg === "--package-manager") {
      packageManager = requirePackageManager(args[index + 1]);
      index += 1;
      continue;
    }
    if (arg.startsWith("--package-manager=")) {
      packageManager = requirePackageManager(arg.slice("--package-manager=".length));
      continue;
    }
    if (arg === "--vite" || arg === "--nuxt") {
      bundlerOverride = arg === "--vite" ? "vite" : "nuxt";
      overrides.bundler = true;
      continue;
    }
    const feature = matchFeatureFlag(arg);
    if (feature !== null) {
      overrides[feature.id] = feature.enabled;
      continue;
    }
    if (arg.startsWith("-")) {
      throw new Error(`Unknown init option: ${arg}`);
    }
    if (root !== null) {
      throw new Error(`Unexpected init argument: ${arg}`);
    }
    root = arg;
  }

  return { root, overrides, bundlerOverride, yes, dryRun, install, packageManager, help };
}

function matchFeatureFlag(arg: string): { id: FeatureId; enabled: boolean } | null {
  for (const id of FEATURE_IDS) {
    if (arg === `--${id}`) {
      return { id, enabled: true };
    }
    if (arg === `--no-${id}`) {
      return { id, enabled: false };
    }
  }
  return null;
}

function requirePackageManager(value: string | undefined): string {
  if (value === undefined || value.startsWith("-")) {
    throw new Error("--package-manager requires a value");
  }
  if (!(PACKAGE_MANAGERS as readonly string[]).includes(value)) {
    throw new Error(
      `Unknown package manager: ${value}. Expected one of ${PACKAGE_MANAGERS.join(", ")}`,
    );
  }
  return value;
}

export function initHelp(): string {
  return `Select, install, and configure Vize in an existing project

Usage: vize init [ROOT] [OPTIONS]

Arguments:
  [ROOT]  Project root containing package.json (default: current directory)

Options:
  -y, --yes                  Accept the detected selection without prompting
      --lint / --no-lint     oxlint plugin
      --vite                 vite plugin (forces the Vite target)
      --nuxt                 nuxt module (forces the Nuxt target)
      --bundler/--no-bundler vite plugin or nuxt module, auto-detected
      --fmt / --no-fmt       vize fmt
      --typecheck            vize check (creates tsconfig.json when missing)
      --no-typecheck
      --editor / --no-editor .vscode/extensions.json recommendation
      --dry-run              Print the plan without writing anything
      --no-install           Write configuration without installing dependencies
      --package-manager <PM> One of ${PACKAGE_MANAGERS.join(", ")} (default: detected)
  -h, --help                 Print help

Without --yes, init prompts. A non-TTY stdin is detected and refused rather than
hung, so CI must pass --yes together with the per-feature flags it wants.
`;
}
