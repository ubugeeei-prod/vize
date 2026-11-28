import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "../../catalog/family-catalog-types.ts";

const root = "src/components/watermark/" as const;

/** Catalog metadata lives with the implementation it describes. */
export const watermarkFamilyCatalogEntry = {
  canonicalName: "watermark",
  title: "Watermark",
  packageSubpath: "./watermark",
  entryFile: `${root}index.ts`,
  sourceFiles: [
    `${root}watermark.vue`,
    `${root}watermark-image.ts`,
    `${root}index.ts`,
    `${root}types.ts`,
  ],
  behaviorContract: `${root}watermark.behavior.md`,
  documentation: `${root}watermark.guide.md`,
  tests: [`${root}tests/behavior.test.ts`],
  typeTests: [`${root}tests/watermark.types.test-d.ts`],
  rendererFixture: "components/watermark/watermark.vue",
  qualityGates: componentQualityGates,
  bundleBudget: {
    exportName: "Watermark",
    retainedSignature: 'data-vize-ui":(?:`watermark`|"watermark"|\'watermark\')',
    maximumJavaScriptGzipBytes: 4_000,
    maximumCssGzipBytes: 0,
  },
  aliases: ["stamp", "repeated label", "draft overlay"],
  upstreamCoverage: ["Ant Design Watermark", "shadcn/ui"],
  dependencies: [],
  maturity: "stable",
  owner: catalogOwner,
} as const satisfies UiFamilyCatalogEntry;
