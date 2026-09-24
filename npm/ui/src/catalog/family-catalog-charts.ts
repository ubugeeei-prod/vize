import {
  catalogOwner,
  stableQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const chartScaleFamilyRoot = "src/families/charts/chart-scale/";
const chartShapeFamilyRoot = "src/families/charts/chart-shape/";

const chartMathQualityGates = [...stableQualityGates, "ssr", "hydration"] as const;

export const chartFamilyCatalog = [
  {
    canonicalName: "chart-scale",
    title: "Chart Scale",
    packageSubpath: "./chart-scale",
    entryFile: `${chartScaleFamilyRoot}chart-scale.ts`,
    sourceFiles: [
      `${chartScaleFamilyRoot}chart-scale.ts`,
      `${chartScaleFamilyRoot}scale-band.ts`,
      `${chartScaleFamilyRoot}scale-continuous.ts`,
      `${chartScaleFamilyRoot}scale-d3-vectors.ts`,
      `${chartScaleFamilyRoot}scale-format.ts`,
      `${chartScaleFamilyRoot}scale-ticks.ts`,
      `${chartScaleFamilyRoot}scale-time-interval.ts`,
      `${chartScaleFamilyRoot}scale-time.ts`,
      `${chartScaleFamilyRoot}scale-types.ts`,
    ],
    behaviorContract: `${chartScaleFamilyRoot}chart-scale.behavior.md`,
    tests: [
      `${chartScaleFamilyRoot}chart-scale.test.ts`,
      `${chartScaleFamilyRoot}chart-scale-ssr.test.ts`,
    ],
    typeTests: [`${chartScaleFamilyRoot}chart-scale.types.test-d.ts`],
    qualityGates: chartMathQualityGates,
    bundleBudget: {
      exportName: "scaleOrdinal",
      retainedSignature: "VIZE_UI_SCALE_EMPTY_RANGE",
      maximumJavaScriptGzipBytes: 900,
      maximumCssGzipBytes: 0,
    },
    aliases: ["scale", "d3 scale", "axis scale", "linear scale", "time scale", "band scale"],
    upstreamCoverage: [
      "d3-scale",
      "d3-array ticks",
      "d3-time",
      "Intl.NumberFormat",
      "Intl.DateTimeFormat",
    ],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
  {
    canonicalName: "chart-shape",
    title: "Chart Shape",
    packageSubpath: "./chart-shape",
    entryFile: `${chartShapeFamilyRoot}chart-shape.ts`,
    sourceFiles: [
      `${chartShapeFamilyRoot}chart-shape.ts`,
      `${chartShapeFamilyRoot}shape-arc.ts`,
      `${chartShapeFamilyRoot}shape-bar.ts`,
      `${chartShapeFamilyRoot}shape-curve.ts`,
      `${chartShapeFamilyRoot}shape-d3-vectors.ts`,
      `${chartShapeFamilyRoot}shape-line.ts`,
      `${chartShapeFamilyRoot}shape-path.ts`,
      `${chartShapeFamilyRoot}shape-stack.ts`,
    ],
    behaviorContract: `${chartShapeFamilyRoot}chart-shape.behavior.md`,
    tests: [
      `${chartShapeFamilyRoot}chart-shape.test.ts`,
      `${chartShapeFamilyRoot}chart-shape-ssr.test.ts`,
    ],
    typeTests: [`${chartShapeFamilyRoot}chart-shape.types.test-d.ts`],
    qualityGates: chartMathQualityGates,
    bundleBudget: {
      exportName: "createPath",
      retainedSignature: "VIZE_UI_PATH_DIGITS",
      maximumJavaScriptGzipBytes: 1_900,
      maximumCssGzipBytes: 0,
    },
    aliases: ["svg path", "d3 shape", "line generator", "area generator", "pie", "stack layout"],
    upstreamCoverage: ["d3-shape", "d3-path"],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
