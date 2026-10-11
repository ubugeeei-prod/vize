/**
 * Current #8371 initializer capability over the unchanged #3956 shape archive.
 * Only generated plans/files change. Every authored input, manager cell,
 * dependency argv and complete clean/broken/repaired diagnostic oracle stays
 * owned by the historical shape modules.
 */
import { PROJECT_SHAPES } from "./smoke-release-init-shapes.mjs";

function configFreeShape(shape) {
  const javascript = shape.id === "vite-vue-js-checkjs";
  const { "vize.config.ts": historicalConfig, ...expectedFiles } = shape.expectedFiles;
  if (historicalConfig === undefined) throw new Error(`${shape.id}: missing historical config`);
  return {
    ...shape,
    features: [
      shape.features[0],
      shape.features[1],
      "  fmt       configured uses project settings and formatter defaults",
      javascript
        ? "  typecheck configured writes tsconfig.json"
        : "  typecheck configured uses tsconfig.json and project settings",
      shape.features[4],
    ],
    reconfiguredDetection: javascript
      ? (manager) => [
          "  framework:       Vite (vite.config.js)",
          `  package manager: ${manager.detectedPackageManager}`,
          "  language:        TypeScript (tsconfig.json)",
          "  lint command:    oxlint",
          "  vize config:     none",
          "  oxlint config:   none",
        ]
      : shape.detection,
    reconfiguredFeatures: [
      shape.reconfiguredFeatures[0],
      shape.reconfiguredFeatures[1],
      "  fmt       unchanged  uses project settings and formatter defaults",
      "  typecheck unchanged  uses tsconfig.json and project settings",
      shape.reconfiguredFeatures[4],
    ],
    createdFiles: shape.createdFiles.filter((filename) => filename !== "vize.config.ts"),
    expectedFiles,
  };
}

export const CONFIG_FREE_PROJECT_SHAPES = Object.fromEntries(
  Object.entries(PROJECT_SHAPES).map(([id, shape]) => [id, configFreeShape(shape)]),
);
