import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const tableFamilyRoot = "src/families/data/table/";

export const dataFamilyCatalog = [
  {
    canonicalName: "table",
    title: "Table",
    packageSubpath: "./table",
    entryFile: `${tableFamilyRoot}table.ts`,
    sourceFiles: [
      `${tableFamilyRoot}table-body.vue`,
      `${tableFamilyRoot}table-caption.vue`,
      `${tableFamilyRoot}table-cell.vue`,
      `${tableFamilyRoot}table-head.vue`,
      `${tableFamilyRoot}table-header.vue`,
      `${tableFamilyRoot}table-row.vue`,
      `${tableFamilyRoot}table.vue`,
      `${tableFamilyRoot}table-contracts.ts`,
      `${tableFamilyRoot}table.ts`,
      `${tableFamilyRoot}table-types.ts`,
    ],
    behaviorContract: `${tableFamilyRoot}table.behavior.md`,
    tests: [`${tableFamilyRoot}table.test.ts`, `${tableFamilyRoot}table-ssr.test.ts`],
    typeTests: [`${tableFamilyRoot}table.types.test-d.ts`],
    rendererFixture: "TableConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Table",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}table",
      maximumJavaScriptGzipBytes: 650,
      maximumCssGzipBytes: 0,
    },
    aliases: ["semantic table", "data table shell", "table primitive", "tabular data"],
    upstreamCoverage: [
      "HTML table element",
      "HTML caption element",
      "HTML table section elements",
      "HTML th scope attribute",
      "shadcn/ui Table",
      "Reka UI Primitive",
    ],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
