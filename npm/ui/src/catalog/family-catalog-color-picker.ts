import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const colorPickerFamilyRoot = "src/families/form/color-picker/";

export const colorPickerFamilyCatalog = [
  {
    canonicalName: "color-picker",
    title: "Color Picker",
    packageSubpath: "./color-picker",
    entryFile: `${colorPickerFamilyRoot}color-picker.ts`,
    sourceFiles: [
      `${colorPickerFamilyRoot}color-picker-root.vue`,
      `${colorPickerFamilyRoot}color-picker-area.vue`,
      `${colorPickerFamilyRoot}color-picker-channel-slider.vue`,
      `${colorPickerFamilyRoot}color-picker-eye-dropper.vue`,
      `${colorPickerFamilyRoot}color-picker-field.vue`,
      `${colorPickerFamilyRoot}color-picker-swatch-group.vue`,
      `${colorPickerFamilyRoot}color-picker-swatch.vue`,
      `${colorPickerFamilyRoot}color-picker.ts`,
      `${colorPickerFamilyRoot}color-picker-color.ts`,
      `${colorPickerFamilyRoot}color-picker-context.ts`,
      `${colorPickerFamilyRoot}color-picker-eye-dropper.ts`,
      `${colorPickerFamilyRoot}color-picker-interaction.ts`,
      `${colorPickerFamilyRoot}color-picker-swatch-context.ts`,
      `${colorPickerFamilyRoot}color-picker-types.ts`,
    ],
    behaviorContract: `${colorPickerFamilyRoot}color-picker.behavior.md`,
    tests: [
      `${colorPickerFamilyRoot}color-picker.test.ts`,
      `${colorPickerFamilyRoot}color-picker-ssr.test.ts`,
      `${colorPickerFamilyRoot}color-picker-color.test.ts`,
      `${colorPickerFamilyRoot}color-picker-interaction.test.ts`,
    ],
    typeTests: [`${colorPickerFamilyRoot}color-picker.types.test-d.ts`],
    rendererFixture: "ColorPickerConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "ColorPicker",
      retainedSignature:
        'data-vize-ui":(?:`color-picker-root`|"color-picker-root"|\'color-picker-root\')',
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 4_600,
      maximumCssGzipBytes: 0,
    },
    aliases: ["color picker", "colour picker", "color input", "hue slider", "eye dropper"],
    upstreamCoverage: [
      "HTML input type=color",
      "React Aria ColorPicker, ColorArea, ColorSlider, ColorSwatchPicker, ColorField",
      "Ark UI ColorPicker",
      "EyeDropper API",
      "CSS Color Module Level 4 rgb()/hsl() syntax",
    ],
    dependencies: ["collection", "composite-navigation", "context", "controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
