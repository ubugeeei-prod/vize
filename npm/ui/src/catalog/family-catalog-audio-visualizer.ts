import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const audioVisualizerFamilyRoot = "src/families/media/audio-visualizer/";

export const audioVisualizerFamilyCatalog = [
  {
    canonicalName: "audio-visualizer",
    title: "Audio Visualizer",
    packageSubpath: "./audio-visualizer",
    entryFile: `${audioVisualizerFamilyRoot}audio-visualizer.ts`,
    sourceFiles: [
      `${audioVisualizerFamilyRoot}audio-visualizer.vue`,
      `${audioVisualizerFamilyRoot}audio-visualizer-bars.vue`,
      `${audioVisualizerFamilyRoot}audio-visualizer.ts`,
      `${audioVisualizerFamilyRoot}audio-visualizer-context.ts`,
      `${audioVisualizerFamilyRoot}audio-visualizer-math.ts`,
      `${audioVisualizerFamilyRoot}audio-visualizer-runtime.ts`,
      `${audioVisualizerFamilyRoot}audio-visualizer-types.ts`,
    ],
    behaviorContract: `${audioVisualizerFamilyRoot}audio-visualizer.behavior.md`,
    tests: [
      `${audioVisualizerFamilyRoot}audio-visualizer.test.ts`,
      `${audioVisualizerFamilyRoot}audio-visualizer-ssr.test.ts`,
    ],
    typeTests: [`${audioVisualizerFamilyRoot}audio-visualizer.types.test-d.ts`],
    rendererFixture: "AudioVisualizerConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "AudioVisualizer",
      retainedSignature: "VIZE_UI_AUDIO_ANALYSER_FFT_SIZE",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 3_600,
      maximumCssGzipBytes: 0,
    },
    aliases: ["audio visualizer", "spectrum analyser", "waveform", "vu meter", "audio level"],
    upstreamCoverage: [
      "Web Audio AnalyserNode",
      "wavesurfer.js",
      "VueUse useDevicesList + AnalyserNode recipes",
    ],
    dependencies: ["context"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
