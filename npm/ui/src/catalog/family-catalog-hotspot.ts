import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const hotspotFamilyRoot = "src/families/media/hotspot/";

export const hotspotFamilyCatalog = [
  {
    canonicalName: "hotspot",
    title: "Hotspot",
    packageSubpath: "./hotspot",
    entryFile: `${hotspotFamilyRoot}hotspot.ts`,
    sourceFiles: [
      `${hotspotFamilyRoot}hotspot-area.vue`,
      `${hotspotFamilyRoot}hotspot-content.vue`,
      `${hotspotFamilyRoot}hotspot-context.ts`,
      `${hotspotFamilyRoot}hotspot-geometry.ts`,
      `${hotspotFamilyRoot}hotspot-image.vue`,
      `${hotspotFamilyRoot}hotspot-marker.vue`,
      `${hotspotFamilyRoot}hotspot-root.vue`,
      `${hotspotFamilyRoot}hotspot-types.ts`,
      `${hotspotFamilyRoot}hotspot.ts`,
    ],
    behaviorContract: `${hotspotFamilyRoot}hotspot.behavior.md`,
    tests: [
      `${hotspotFamilyRoot}hotspot.test.ts`,
      `${hotspotFamilyRoot}hotspot-ssr.test.ts`,
      `${hotspotFamilyRoot}hotspot-geometry.test.ts`,
    ],
    typeTests: [`${hotspotFamilyRoot}hotspot.types.test-d.ts`],
    rendererFixture: "HotspotConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "HotspotRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}hotspot-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 2_650,
      maximumCssGzipBytes: 0,
    },
    aliases: ["hotspot", "image map", "annotated image", "shoppable image", "product tags"],
    upstreamCoverage: ["HTML map/area", "WAI-ARIA Disclosure", "Popover API"],
    dependencies: ["context", "controllable-state", "id", "popover"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
