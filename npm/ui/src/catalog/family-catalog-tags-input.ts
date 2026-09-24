import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const tagsInputFamilyRoot = "src/families/form/tags-input/";

export const tagsInputFamilyCatalog = [
  {
    canonicalName: "tags-input",
    title: "Tags Input",
    packageSubpath: "./tags-input",
    entryFile: `${tagsInputFamilyRoot}tags-input.ts`,
    sourceFiles: [
      `${tagsInputFamilyRoot}tags-input-context.ts`,
      `${tagsInputFamilyRoot}tags-input-input.vue`,
      `${tagsInputFamilyRoot}tags-input-item-delete.vue`,
      `${tagsInputFamilyRoot}tags-input-item-text.vue`,
      `${tagsInputFamilyRoot}tags-input-item.vue`,
      `${tagsInputFamilyRoot}tags-input-root.vue`,
      `${tagsInputFamilyRoot}tags-input-types.ts`,
      `${tagsInputFamilyRoot}tags-input-value.ts`,
      `${tagsInputFamilyRoot}tags-input.ts`,
    ],
    behaviorContract: `${tagsInputFamilyRoot}tags-input.behavior.md`,
    tests: [
      `${tagsInputFamilyRoot}tags-input.test.ts`,
      `${tagsInputFamilyRoot}tags-input-ssr.test.ts`,
    ],
    typeTests: [`${tagsInputFamilyRoot}tags-input.types.test-d.ts`],
    rendererFixture: "TagsInputConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "TagsInputRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}tags-input",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 4_300,
      maximumCssGzipBytes: 0,
    },
    aliases: ["tags input", "chips input", "token field", "tag editor"],
    upstreamCoverage: ["Reka UI TagsInput", "Ark UI Tags Input", "Mantine TagsInput"],
    dependencies: ["context", "controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
