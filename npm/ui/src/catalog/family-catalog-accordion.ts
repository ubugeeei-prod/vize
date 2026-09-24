import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const accordionFamilyRoot = "src/families/disclosure/accordion/";

export const accordionFamilyCatalog = [
  {
    canonicalName: "accordion",
    title: "Accordion",
    packageSubpath: "./accordion",
    entryFile: `${accordionFamilyRoot}accordion.ts`,
    sourceFiles: [
      `${accordionFamilyRoot}accordion-content.vue`,
      `${accordionFamilyRoot}accordion-context.ts`,
      `${accordionFamilyRoot}accordion-header.vue`,
      `${accordionFamilyRoot}accordion-item.vue`,
      `${accordionFamilyRoot}accordion-root.vue`,
      `${accordionFamilyRoot}accordion-trigger.vue`,
      `${accordionFamilyRoot}accordion-types.ts`,
      `${accordionFamilyRoot}accordion-value.ts`,
      `${accordionFamilyRoot}accordion.ts`,
    ],
    behaviorContract: `${accordionFamilyRoot}accordion.behavior.md`,
    tests: [
      `${accordionFamilyRoot}accordion.test.ts`,
      `${accordionFamilyRoot}accordion-ssr.test.ts`,
    ],
    typeTests: [`${accordionFamilyRoot}accordion.types.test-d.ts`],
    rendererFixture: "AccordionConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "AccordionRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}accordion-root",
      allowedRetainedFamilies: ["collection", "context", "controllable-state"],
      maximumJavaScriptGzipBytes: 5_750,
      maximumCssGzipBytes: 0,
    },
    aliases: ["accordion", "expansion panel", "disclosure group", "faq"],
    upstreamCoverage: [
      "WAI-ARIA Accordion",
      "Radix UI Accordion",
      "Reka UI Accordion",
      "HTML hidden=until-found",
    ],
    dependencies: ["collapsible", "collection", "context", "controllable-state", "id", "primitive"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
