export const devCacheReporterFilename = "vize-parcel-dev-cache.cjs";

// This file runs inside the normal consumer as a CommonJS Parcel plugin.
export const devCacheReporterSource =
  'module.exports = require("@vizejs/parcel-transformer/dev-cache");\n';

export function parcelHostConfig() {
  return {
    extends: "@parcel/config-default",
    reporters: [`./${devCacheReporterFilename}`, "./native-trace.cjs", "..."],
    transformers: {
      "*.vue": ["@vizejs/parcel-transformer"],
      "vize-component:*.vue": ["@vizejs/parcel-transformer"],
      "*.css": ["@vizejs/parcel-transformer/style", "..."],
    },
  };
}
