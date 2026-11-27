import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "../../catalog/family-catalog-types.ts";

const root = "src/components/color-wheel/" as const;

/** Catalog metadata lives with the implementation it describes. */
export const colorWheelFamilyCatalogEntry = {
  canonicalName: "color-wheel",
  title: "Color Wheel",
  packageSubpath: "./color-wheel",
  entryFile: `${root}index.ts`,
  sourceFiles: [
    `${root}color-wheel.vue`,
    `${root}color-wheel-geometry.ts`,
    `${root}index.ts`,
    `${root}types.ts`,
  ],
  behaviorContract: `${root}color-wheel.behavior.md`,
  documentation: `${root}color-wheel.guide.md`,
  tests: [`${root}tests/behavior.test.ts`],
  typeTests: [`${root}tests/color-wheel.types.test-d.ts`],
  rendererFixture: "components/color-wheel/color-wheel.vue",
  qualityGates: componentQualityGates,
  bundleBudget: {
    exportName: "ColorWheel",
    retainedSignature: 'data-vize-ui":(?:`color-wheel`|"color-wheel"|\'color-wheel\')',
    maximumJavaScriptGzipBytes: 6_000,
    maximumCssGzipBytes: 0,
  },
  aliases: ["hue wheel", "color hue", "polar hue"],
  upstreamCoverage: ["Adobe React Aria ColorWheel", "React Spectrum ColorWheel"],
  dependencies: [],
  maturity: "stable",
  owner: catalogOwner,
} as const satisfies UiFamilyCatalogEntry;
