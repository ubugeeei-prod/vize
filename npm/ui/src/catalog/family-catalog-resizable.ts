import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const resizableFamilyRoot = "src/families/layout/resizable/";

export const resizableFamilyCatalog = [
  {
    canonicalName: "resizable",
    title: "Resizable",
    packageSubpath: "./resizable",
    entryFile: `${resizableFamilyRoot}resizable.ts`,
    sourceFiles: [
      `${resizableFamilyRoot}resizable-context.ts`,
      `${resizableFamilyRoot}resizable-geometry.ts`,
      `${resizableFamilyRoot}resizable-handle.vue`,
      `${resizableFamilyRoot}resizable-root.vue`,
      `${resizableFamilyRoot}resizable-types.ts`,
      `${resizableFamilyRoot}resizable.ts`,
    ],
    behaviorContract: `${resizableFamilyRoot}resizable.behavior.md`,
    tests: [
      `${resizableFamilyRoot}resizable.test.ts`,
      `${resizableFamilyRoot}resizable-ssr.test.ts`,
    ],
    typeTests: [`${resizableFamilyRoot}resizable.types.test-d.ts`],
    rendererFixture: "ResizableConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "ResizableRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}resizable-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 3_000,
      maximumCssGzipBytes: 0,
    },
    aliases: ["resizable", "resize handle", "resizable box", "element resizer"],
    upstreamCoverage: [
      "WAI-ARIA window splitter (separator) pattern",
      "CSS resize property",
      "re-resizable",
    ],
    dependencies: ["context", "controllable-state", "id", "move"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
