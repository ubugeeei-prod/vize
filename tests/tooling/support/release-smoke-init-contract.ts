export const SHAPE_KEYS = [
  "addedScripts",
  "createdFiles",
  "detection",
  "expectedDevDependencies",
  "expectedFiles",
  "expectedScripts",
  "features",
  "initFlags",
  "initialAbsentFiles",
  "plannedDependencies",
  "reconfiguredDetection",
  "reconfiguredFeatures",
  "requires",
  "updatedFiles",
];

/** Code-unit order, matching the driver's own comparison of installed names. */
export const byCodeUnit = (left: string, right: string): number =>
  left < right ? -1 : left > right ? 1 : 0;

export const MANAGER_KEYS = [
  "bootstrapArgs",
  "corsaInstallCommand",
  "detectedPackageManager",
  "environment",
  "installArgs",
  "installFlags",
  "lockfile",
  "projectFiles",
  "redirect",
  "redirectPlannedDependencies",
  "runScriptArgs",
];

export const COREPACK_MANAGER_SPECS = {
  pnpm: "pnpm@11.21.0",
  yarn: "yarn@4.9.2",
};
