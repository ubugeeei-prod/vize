import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/layout/avatar/";

export const avatarLayoutFamilyCatalog = [
  {
    canonicalName: "avatar",
    title: "Avatar",
    packageSubpath: "./avatar",
    entryFile: `${familyRoot}avatar.ts`,
    sourceFiles: [
      `${familyRoot}avatar.vue`,
      `${familyRoot}avatar.ts`,
      `${familyRoot}avatar-types.ts`,
    ],
    behaviorContract: `${familyRoot}avatar.behavior.md`,
    tests: [`${familyRoot}avatar.test.ts`, `${familyRoot}avatar-ssr.test.ts`],
    typeTests: [`${familyRoot}avatar.types.test-d.ts`],
    rendererFixture: "AvatarConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Avatar",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}avatar",
      maximumJavaScriptGzipBytes: 1_450,
      maximumCssGzipBytes: 0,
    },
    aliases: ["profile image", "user avatar", "presence avatar", "fallback avatar"],
    upstreamCoverage: ["HTML img element", "native image loading", "native image decoding"],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
