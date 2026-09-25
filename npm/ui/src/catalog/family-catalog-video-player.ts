import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const videoPlayerFamilyRoot = "src/families/media/video-player/";

export const videoPlayerFamilyCatalog = [
  {
    canonicalName: "video-player",
    title: "Video Player",
    packageSubpath: "./video-player",
    entryFile: `${videoPlayerFamilyRoot}video-player.ts`,
    sourceFiles: [
      `${videoPlayerFamilyRoot}video-player-fullscreen-button.vue`,
      `${videoPlayerFamilyRoot}video-player-picture-in-picture-button.vue`,
      `${videoPlayerFamilyRoot}video-player-types.ts`,
      `${videoPlayerFamilyRoot}video-player-video.vue`,
      `${videoPlayerFamilyRoot}video-player.ts`,
    ],
    behaviorContract: `${videoPlayerFamilyRoot}video-player.behavior.md`,
    tests: [
      `${videoPlayerFamilyRoot}video-player.test.ts`,
      `${videoPlayerFamilyRoot}video-player-ssr.test.ts`,
    ],
    typeTests: [`${videoPlayerFamilyRoot}video-player.types.test-d.ts`],
    rendererFixture: "VideoPlayerConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "VideoPlayerVideo",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}video-player-video",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 2_500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["video player", "video", "video controls", "picture in picture", "fullscreen video"],
    upstreamCoverage: [
      "HTML video",
      "Fullscreen API",
      "Picture-in-Picture API",
      "Vidstack Player",
      "Media Chrome",
    ],
    dependencies: ["context", "media-player"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
