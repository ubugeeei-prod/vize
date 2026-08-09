import type { ProjectDetection } from "./detect.js";
import { discoveredOxlintConfig, unreadOxlintConfig } from "./lint-target.js";

export const FEATURE_IDS = ["lint", "bundler", "fmt", "typecheck", "editor"] as const;

export type FeatureId = (typeof FEATURE_IDS)[number];

export type FeatureSelection = Readonly<Record<FeatureId, boolean>>;

export interface FeatureOffer {
  readonly id: FeatureId;
  /** Label shown in the prompt and in the plan. Reflects what detection found. */
  readonly label: string;
  /** False when the project cannot support the feature at all. */
  readonly available: boolean;
  /** True when the project already has this feature wired up. */
  readonly configured: boolean;
  /** Why the feature is unavailable or already configured. Empty when neither. */
  readonly note: string;
  readonly defaultSelected: boolean;
}

/**
 * Turns detection into the five offers `init` presents.
 *
 * Already-configured features stay selected by default so a re-run is a no-op
 * the user can confirm rather than a set of boxes they have to re-tick.
 */
export function offerFeatures(detection: ProjectDetection): readonly FeatureOffer[] {
  return [
    lintOffer(detection),
    bundlerOffer(detection),
    fmtOffer(detection),
    typecheckOffer(detection),
    editorOffer(detection),
  ];
}

/** Selection implied by detection alone, used by `--yes` and as the prompt default. */
export function defaultSelection(offers: readonly FeatureOffer[]): FeatureSelection {
  const selection: Record<FeatureId, boolean> = {
    lint: false,
    bundler: false,
    fmt: false,
    typecheck: false,
    editor: false,
  };
  for (const offer of offers) {
    selection[offer.id] = offer.defaultSelected;
  }
  return selection;
}

function lintOffer(detection: ProjectDetection): FeatureOffer {
  const configured = detection.usesVitePlus
    ? detection.hasVitePlusLintBlock
    : discoveredOxlintConfig(detection) !== null;
  const unread = unreadOxlintConfig(detection);
  const label = detection.usesVitePlus
    ? "oxlint plugin (vp lint reads the `lint` block in the Vite config)"
    : "oxlint plugin (the oxlint binary reads oxlint.config.ts)";
  return {
    id: "lint",
    label,
    available: true,
    configured,
    note: configured
      ? "already configured"
      : unread === null
        ? ""
        : `${unread} exists but oxlint never reads it (#3474)`,
    defaultSelected: true,
  };
}

function bundlerOffer(detection: ProjectDetection): FeatureOffer {
  if (detection.framework === "nuxt") {
    return {
      id: "bundler",
      label: "nuxt module (@vizejs/nuxt)",
      available: detection.nuxtConfig !== null,
      configured: detection.hasVizeNuxtModule,
      note: detection.hasVizeNuxtModule
        ? "already configured"
        : detection.nuxtConfig === null
          ? "no nuxt.config file to add @vizejs/nuxt to"
          : "",
      defaultSelected: detection.nuxtConfig !== null,
    };
  }
  if (detection.framework === "vite") {
    const single = detection.viteConfigs.length === 1;
    return {
      id: "bundler",
      label: "vite plugin (@vizejs/vite-plugin)",
      available: single,
      configured: detection.hasVizeVitePlugin,
      note: detection.hasVizeVitePlugin
        ? "already configured"
        : single
          ? ""
          : `several Vite configs (${detection.viteConfigs.join(", ")})`,
      defaultSelected: single,
    };
  }
  return {
    id: "bundler",
    label: "vite plugin or nuxt module",
    available: false,
    configured: false,
    note: "no vite.config or nuxt.config found; the other features work without one",
    defaultSelected: false,
  };
}

function fmtOffer(detection: ProjectDetection): FeatureOffer {
  const configured = detection.vizeConfig !== null && "vize:fmt" in detection.scripts;
  return {
    id: "fmt",
    label: "fmt (vize fmt)",
    available: true,
    configured,
    note: configured ? "already configured" : "",
    defaultSelected: true,
  };
}

function typecheckOffer(detection: ProjectDetection): FeatureOffer {
  const configured =
    detection.tsconfig !== null &&
    detection.vizeConfig !== null &&
    "vize:check" in detection.scripts;
  return {
    id: "typecheck",
    label: "typecheck (vize check)",
    available: true,
    configured,
    note: configured
      ? "already configured"
      : detection.tsconfig === null
        ? "creates tsconfig.json"
        : "",
    defaultSelected: true,
  };
}

function editorOffer(detection: ProjectDetection): FeatureOffer {
  return {
    id: "editor",
    label: "editor extension (.vscode/extensions.json recommendation)",
    available: true,
    configured: detection.vscodeRecommendsVize,
    note: detection.vscodeRecommendsVize ? "already recommended" : "",
    defaultSelected: true,
  };
}
