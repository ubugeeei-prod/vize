import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

export const navigationFamilyCatalog = [
  {
    canonicalName: "breadcrumb",
    title: "Breadcrumb",
    packageSubpath: "./breadcrumb",
    entryFile: "src/breadcrumb.ts",
    sourceFiles: [
      "src/breadcrumb.vue",
      "src/breadcrumb-item.vue",
      "src/breadcrumb-link.vue",
      "src/breadcrumb-list.vue",
      "src/breadcrumb-separator.vue",
      "src/breadcrumb.ts",
      "src/breadcrumb-types.ts",
    ],
    behaviorContract: "src/breadcrumb.behavior.md",
    tests: ["src/breadcrumb.test.ts", "src/breadcrumb-ssr.test.ts"],
    typeTests: ["src/breadcrumb.types.test-d.ts"],
    rendererFixture: "BreadcrumbConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Breadcrumb",
      retainedSignature: 'data-vize-ui":(?:`breadcrumb`|"breadcrumb"|\'breadcrumb\')',
      maximumJavaScriptGzipBytes: 1_725,
      maximumCssGzipBytes: 0,
    },
    aliases: ["breadcrumb navigation", "route breadcrumb", "path trail", "hierarchy trail"],
    upstreamCoverage: [
      "HTML nav landmark",
      "WAI-ARIA breadcrumb pattern",
      "React Aria Breadcrumbs",
    ],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
