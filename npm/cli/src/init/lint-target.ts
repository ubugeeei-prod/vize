import type { ProjectDetection } from "./detect.js";
import { DISCOVERED_OXLINT_CONFIG_FILES } from "../setup/config.js";
import { canInjectViteLint, hasTopLevelKey } from "./edit-config.js";
import { VITE_LINT_MERGE_SNIPPET, VITE_LINT_SNIPPET } from "./templates.js";

export { DISCOVERED_OXLINT_CONFIG_FILES } from "../setup/config.js";

/**
 * Oxlint config filenames the `oxlint` binary actually auto-discovers.
 *
 * Verified against oxlint 1.64: `.oxlintrc.json`, `.oxlintrc.jsonc` and
 * `oxlint.config.ts` are read; `oxlint.config.mts`, `.js`, `.mjs`, `.cjs` and
 * `.cts` produce a run byte-identical to having no config at all.
 */
/** Filename `init` writes when the `oxlint` binary is the lint entry point. */
export const INIT_OXLINT_CONFIG_FILE = "oxlint.config.ts";

/**
 * Where a project's Oxlint configuration has to live to be read.
 *
 * `vp lint` and `vp check` read the `lint` block of `vite.config.ts` and never
 * read `.oxlintrc.json`; the `oxlint` and `oxlint-vize` binaries read
 * `.oxlintrc.json` and never read `vite.config.ts`. Writing the wrong one leaves
 * a project that looks configured, reports zero `vize/*` diagnostics and exits
 * `0` -- the defect #3389 recorded and #3407 fixed. Every branch below therefore
 * follows the command the project will actually run, not the file that is
 * easiest to write.
 */
export type LintTargetKind =
  /** Vite+ project: the `lint` block in the Vite config is the only readable place. */
  | "vite-plus"
  /** No Vite+: the `oxlint` binary reads its own config file. */
  | "oxlint"
  /** Both entry points are in use; both files get written from one settings object. */
  | "both"
  /** Vite+ project whose Vite config cannot be edited safely. Nothing is written. */
  | "manual";

export interface LintTarget {
  readonly kind: LintTargetKind;
  /** Vite config to receive the `lint` block, when one can be edited. */
  readonly viteConfig: string | null;
  /** Oxlint config file to write, when the `oxlint` binary is an entry point. */
  readonly oxlintConfig: string | null;
  /** Existing Oxlint config left untouched, if any. */
  readonly preservedOxlintConfig: string | null;
  /** Why this target was chosen. Always shown to the user. */
  readonly reason: string;
  /**
   * Set when the project needs a Vite+ `lint` block that `init` will not write.
   * The caller must print the snippet and must not claim lint is configured.
   */
  readonly blockedReason: string | null;
  /** Snippet to paste when `blockedReason` is set. */
  readonly blockedSnippet: string | null;
}

export interface LintTargetInput {
  readonly detection: ProjectDetection;
  /** Source of the single Vite config, or `null` when there is not exactly one. */
  readonly viteSource: string | null;
}

/**
 * Chooses which Oxlint configuration file(s) the project needs.
 *
 * The `oxlint` binary is treated as an entry point whenever the project already
 * carries a discovered Oxlint config or runs `oxlint` from a script. A Vite+
 * project that also does either gets both files, generated from the same preset
 * and help level, because keeping one of them silently stale is the same class
 * of bug as writing the wrong one.
 */
export function resolveLintTarget(input: LintTargetInput): LintTarget {
  const { detection } = input;
  const existing = discoveredOxlintConfig(detection);
  const runsOxlintBinary = existing !== null || hasOxlintScript(detection);

  if (!detection.usesVitePlus) {
    return {
      kind: "oxlint",
      viteConfig: null,
      oxlintConfig: existing === null ? INIT_OXLINT_CONFIG_FILE : null,
      preservedOxlintConfig: existing,
      reason:
        "no Vite+ detected, so `oxlint` is the lint entry point and reads " +
        `${existing ?? INIT_OXLINT_CONFIG_FILE}`,
      blockedReason: null,
      blockedSnippet: null,
    };
  }

  const viteConfig = detection.viteConfigs.length === 1 ? detection.viteConfigs[0]! : null;
  const injectable = input.viteSource !== null && canInjectViteLint(input.viteSource);
  if (!detection.hasVitePlusLintBlock && !injectable) {
    const blocked = describeBlocked(detection, input.viteSource);
    return {
      kind: "manual",
      viteConfig: null,
      oxlintConfig: null,
      preservedOxlintConfig: existing,
      reason: "Vite+ detected, so `vp lint` reads the `lint` block in the Vite config",
      blockedReason: blocked.reason,
      blockedSnippet: blocked.snippet,
    };
  }

  if (!runsOxlintBinary) {
    return {
      kind: "vite-plus",
      viteConfig,
      oxlintConfig: null,
      preservedOxlintConfig: null,
      reason:
        "Vite+ detected, so `vp lint` reads the `lint` block in " +
        `${viteConfig ?? "the Vite config"} and never reads .oxlintrc.json`,
      blockedReason: null,
      blockedSnippet: null,
    };
  }

  return {
    kind: "both",
    viteConfig,
    oxlintConfig: existing === null ? INIT_OXLINT_CONFIG_FILE : null,
    preservedOxlintConfig: existing,
    reason:
      "Vite+ and the `oxlint` binary are both in use, so the `lint` block in " +
      `${viteConfig ?? "the Vite config"} and ${existing ?? INIT_OXLINT_CONFIG_FILE} ` +
      "are written from the same preset",
    blockedReason: null,
    blockedSnippet: null,
  };
}

/**
 * The existing Oxlint config, restricted to names Oxlint actually reads.
 *
 * A project holding only `oxlint.config.mjs` is deliberately treated as having
 * no Oxlint config, because that is how Oxlint treats it.
 */
export function discoveredOxlintConfig(detection: ProjectDetection): string | null {
  const existing = detection.oxlintConfig;
  if (existing === null) {
    return null;
  }
  return (DISCOVERED_OXLINT_CONFIG_FILES as readonly string[]).includes(existing) ? existing : null;
}

/** An Oxlint config file that is present but which Oxlint will never read. */
export function unreadOxlintConfig(detection: ProjectDetection): string | null {
  const existing = detection.oxlintConfig;
  if (existing === null || discoveredOxlintConfig(detection) !== null) {
    return null;
  }
  return existing;
}

function hasOxlintScript(detection: ProjectDetection): boolean {
  return Object.values(detection.scripts).some((command) =>
    /(?:^|[\s&|;])oxlint(?:-vize)?(?:\s|$)/u.test(command),
  );
}

/**
 * Why the `lint` block will not be written.
 *
 * The message is the whole value of a blocked result, so it names the specific
 * obstacle instead of a generic "could not edit". An existing `lint` block in
 * particular is a merge the user has to make, not a failure of the file.
 */
function describeBlocked(
  detection: ProjectDetection,
  viteSource: string | null,
): { readonly reason: string; readonly snippet: string } {
  if (detection.viteConfigs.length === 0) {
    return { reason: "no vite.config file to hold the `lint` block", snippet: VITE_LINT_SNIPPET };
  }
  if (detection.viteConfigs.length > 1) {
    return {
      reason: `several Vite configs (${detection.viteConfigs.join(", ")}), so the target is ambiguous`,
      snippet: VITE_LINT_SNIPPET,
    };
  }
  const filename = detection.viteConfigs[0]!;
  if (viteSource !== null && hasTopLevelKey(viteSource, "lint")) {
    return {
      reason:
        `${filename} already has a \`lint\` block and merging into it would risk dropping ` +
        "settings, so spread createVizeLintConfig() into it by hand",
      snippet: VITE_LINT_MERGE_SNIPPET,
    };
  }
  return {
    reason: `${filename} is not a single plain defineConfig({ ... }) call`,
    snippet: VITE_LINT_SNIPPET,
  };
}
