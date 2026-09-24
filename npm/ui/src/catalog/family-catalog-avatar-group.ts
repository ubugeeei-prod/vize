import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const avatarGroupFamilyRoot = "src/families/layout/avatar-group/";

export const avatarGroupFamilyCatalog = [
  {
    canonicalName: "avatar-group",
    title: "Avatar Group",
    packageSubpath: "./avatar-group",
    entryFile: `${avatarGroupFamilyRoot}avatar-group.ts`,
    sourceFiles: [
      `${avatarGroupFamilyRoot}avatar-group-overflow.vue`,
      `${avatarGroupFamilyRoot}avatar-group-state.ts`,
      `${avatarGroupFamilyRoot}avatar-group-types.ts`,
      `${avatarGroupFamilyRoot}avatar-group.ts`,
      `${avatarGroupFamilyRoot}avatar-group.vue`,
    ],
    behaviorContract: `${avatarGroupFamilyRoot}avatar-group.behavior.md`,
    tests: [
      `${avatarGroupFamilyRoot}avatar-group.test.ts`,
      `${avatarGroupFamilyRoot}avatar-group-ssr.test.ts`,
    ],
    typeTests: [`${avatarGroupFamilyRoot}avatar-group.types.test-d.ts`],
    rendererFixture: "AvatarGroupConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "AvatarGroup",
      retainedSignature: 'data-vize-ui":(?:`avatar-group`|"avatar-group"|\'avatar-group\')',
      allowedRetainedFamilies: ["avatar"],
      maximumJavaScriptGzipBytes: 2_600,
      maximumCssGzipBytes: 0,
    },
    aliases: ["avatar group", "avatar stack", "facepile", "people list"],
    upstreamCoverage: ["Chakra UI AvatarGroup", "MUI AvatarGroup", "Mantine Avatar.Group"],
    dependencies: ["avatar"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
