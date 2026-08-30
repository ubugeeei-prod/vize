import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

export const avatarLayoutFamilyCatalog = [
  {
    canonicalName: "avatar",
    title: "Avatar",
    packageSubpath: "./avatar",
    entryFile: "src/avatar.ts",
    sourceFiles: ["src/avatar.vue", "src/avatar.ts", "src/avatar-types.ts"],
    behaviorContract: "src/avatar.behavior.md",
    tests: ["src/avatar.test.ts", "src/avatar-ssr.test.ts"],
    typeTests: ["src/avatar.types.test-d.ts"],
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
