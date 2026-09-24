import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const audioPlayerFamilyRoot = "src/families/media/audio-player/";

export const audioPlayerFamilyCatalog = [
  {
    canonicalName: "audio-player",
    title: "Audio Player",
    packageSubpath: "./audio-player",
    entryFile: `${audioPlayerFamilyRoot}audio-player.ts`,
    sourceFiles: [
      `${audioPlayerFamilyRoot}audio-player-audio.vue`,
      `${audioPlayerFamilyRoot}audio-player.ts`,
    ],
    behaviorContract: `${audioPlayerFamilyRoot}audio-player.behavior.md`,
    tests: [
      `${audioPlayerFamilyRoot}audio-player.test.ts`,
      `${audioPlayerFamilyRoot}audio-player-ssr.test.ts`,
    ],
    typeTests: [`${audioPlayerFamilyRoot}audio-player.types.test-d.ts`],
    rendererFixture: "AudioPlayerConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "AudioPlayerAudio",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}audio-player-audio",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 2_300,
      maximumCssGzipBytes: 0,
    },
    aliases: ["audio player", "audio", "podcast player", "music player"],
    upstreamCoverage: ["HTML audio", "Vidstack Player", "Media Chrome"],
    dependencies: ["context", "media-player"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
