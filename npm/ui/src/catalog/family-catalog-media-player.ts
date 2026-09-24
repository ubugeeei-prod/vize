import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const mediaPlayerFamilyRoot = "src/families/media/media-player/";

export const mediaPlayerFamilyCatalog = [
  {
    canonicalName: "media-player",
    title: "Media Player",
    packageSubpath: "./media-player",
    entryFile: `${mediaPlayerFamilyRoot}media-player.ts`,
    sourceFiles: [
      `${mediaPlayerFamilyRoot}media-player-captions-button.vue`,
      `${mediaPlayerFamilyRoot}media-player-context.ts`,
      `${mediaPlayerFamilyRoot}media-player-format.ts`,
      `${mediaPlayerFamilyRoot}media-player-loading-indicator.vue`,
      `${mediaPlayerFamilyRoot}media-player-mute-button.vue`,
      `${mediaPlayerFamilyRoot}media-player-platform.ts`,
      `${mediaPlayerFamilyRoot}media-player-play-button.vue`,
      `${mediaPlayerFamilyRoot}media-player-playback-rate-button.vue`,
      `${mediaPlayerFamilyRoot}media-player-root.vue`,
      `${mediaPlayerFamilyRoot}media-player-seek-slider.vue`,
      `${mediaPlayerFamilyRoot}media-player-time-display.vue`,
      `${mediaPlayerFamilyRoot}media-player-types.ts`,
      `${mediaPlayerFamilyRoot}media-player-volume-slider.vue`,
      `${mediaPlayerFamilyRoot}media-player.ts`,
    ],
    behaviorContract: `${mediaPlayerFamilyRoot}media-player.behavior.md`,
    tests: [
      `${mediaPlayerFamilyRoot}media-player.test.ts`,
      `${mediaPlayerFamilyRoot}media-player-ssr.test.ts`,
      `${mediaPlayerFamilyRoot}media-player-format.test.ts`,
    ],
    typeTests: [`${mediaPlayerFamilyRoot}media-player.types.test-d.ts`],
    rendererFixture: "MediaPlayerConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "MediaPlayerRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}media-player-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 5_350,
      maximumCssGzipBytes: 0,
    },
    aliases: ["media player", "media controls", "player controls", "seek bar", "scrubber"],
    upstreamCoverage: [
      "HTML media elements",
      "WAI-ARIA Slider",
      "Vidstack Player",
      "Media Chrome",
      "Radix-style compound controls",
    ],
    dependencies: ["context", "controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
