import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const sidebarFamilyRoot = "src/families/layout/sidebar/";

export const sidebarFamilyCatalog = [
  {
    canonicalName: "sidebar",
    title: "Sidebar",
    packageSubpath: "./sidebar",
    entryFile: `${sidebarFamilyRoot}sidebar.ts`,
    sourceFiles: [
      `${sidebarFamilyRoot}sidebar-content.vue`,
      `${sidebarFamilyRoot}sidebar-context.ts`,
      `${sidebarFamilyRoot}sidebar-footer.vue`,
      `${sidebarFamilyRoot}sidebar-group-label.vue`,
      `${sidebarFamilyRoot}sidebar-group.vue`,
      `${sidebarFamilyRoot}sidebar-header.vue`,
      `${sidebarFamilyRoot}sidebar-inset.vue`,
      `${sidebarFamilyRoot}sidebar-provider.vue`,
      `${sidebarFamilyRoot}sidebar-rail.vue`,
      `${sidebarFamilyRoot}sidebar-root.vue`,
      `${sidebarFamilyRoot}sidebar-trigger.vue`,
      `${sidebarFamilyRoot}sidebar-types.ts`,
      `${sidebarFamilyRoot}sidebar.ts`,
    ],
    behaviorContract: `${sidebarFamilyRoot}sidebar.behavior.md`,
    tests: [`${sidebarFamilyRoot}sidebar.test.ts`, `${sidebarFamilyRoot}sidebar-ssr.test.ts`],
    typeTests: [`${sidebarFamilyRoot}sidebar.types.test-d.ts`],
    rendererFixture: "SidebarConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "SidebarProvider",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}sidebar-provider",
      allowedRetainedFamilies: ["context", "controllable-state", "shortcut"],
      maximumJavaScriptGzipBytes: 6_000,
      maximumCssGzipBytes: 0,
    },
    aliases: ["sidebar", "app sidebar", "context panel", "navigation rail", "off-canvas nav"],
    upstreamCoverage: ["shadcn/ui Sidebar", "HTML aside landmark", "WAI-ARIA dialog pattern"],
    dependencies: ["context", "controllable-state", "dialog", "id", "primitive", "shortcut"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
