import {
  catalogOwner,
  stableQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const foundationFamilyRoot = "src/families/foundations/";
const forwardingFamilyRoot = `${foundationFamilyRoot}forwarding/`;
const polymorphicFamilyRoot = `${foundationFamilyRoot}polymorphic/`;
const slotUtilsFamilyRoot = `${foundationFamilyRoot}slot-utils/`;
const variantsFamilyRoot = `${foundationFamilyRoot}variants/`;

/** Typed authoring helpers for building wrapper components on top of the library. */
export const helperFamilyCatalog = [
  {
    canonicalName: "forwarding",
    title: "Prop, Emit, and Expose Forwarding",
    packageSubpath: "./forwarding",
    entryFile: `${forwardingFamilyRoot}forwarding.ts`,
    sourceFiles: [`${forwardingFamilyRoot}forwarding.ts`],
    behaviorContract: `${forwardingFamilyRoot}forwarding.behavior.md`,
    tests: [`${forwardingFamilyRoot}forwarding.test.ts`],
    typeTests: [`${forwardingFamilyRoot}forwarding.types.test-d.ts`],
    qualityGates: [...stableQualityGates, "ssr", "hydration"],
    bundleBudget: {
      exportName: "useForwardPropsEmits",
      retainedSignature: "\\/-\\(\\\\w\\)\\/gu",
      maximumJavaScriptGzipBytes: 700,
      maximumCssGzipBytes: 0,
    },
    aliases: ["forward props", "forward emits", "forward expose", "wrapper component helpers"],
    upstreamCoverage: [
      "Reka UI useForwardProps",
      "Reka UI useEmitAsProps",
      "Reka UI useForwardPropsEmits",
      "Reka UI useForwardExpose",
    ],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
  {
    canonicalName: "polymorphic",
    title: "Typed Polymorphic Rendering",
    packageSubpath: "./polymorphic",
    entryFile: `${polymorphicFamilyRoot}polymorphic.ts`,
    sourceFiles: [`${polymorphicFamilyRoot}polymorphic.ts`],
    behaviorContract: `${polymorphicFamilyRoot}polymorphic.behavior.md`,
    tests: [`${polymorphicFamilyRoot}polymorphic.test.ts`],
    typeTests: [`${polymorphicFamilyRoot}polymorphic.types.test-d.ts`],
    qualityGates: [...stableQualityGates, "ssr", "hydration"],
    bundleBudget: {
      exportName: "renderPolymorphic",
      retainedSignature: "VIZE_UI_POLYMORPHIC_AS",
      maximumJavaScriptGzipBytes: 700,
      maximumCssGzipBytes: 0,
    },
    aliases: ["as prop", "polymorphic component", "element type inference"],
    upstreamCoverage: ["Radix Polymorphic", "Ark UI polymorphic factory", "Reka UI Primitive as"],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
  {
    canonicalName: "slot-utils",
    title: "Slot and Prop Merging Utilities",
    packageSubpath: "./slot-utils",
    entryFile: `${slotUtilsFamilyRoot}slot-utils.ts`,
    sourceFiles: [`${slotUtilsFamilyRoot}slot-utils.ts`],
    behaviorContract: `${slotUtilsFamilyRoot}slot-utils.behavior.md`,
    tests: [`${slotUtilsFamilyRoot}slot-utils.test.ts`],
    typeTests: [`${slotUtilsFamilyRoot}slot-utils.types.test-d.ts`],
    qualityGates: [...stableQualityGates, "ssr", "hydration"],
    bundleBudget: {
      exportName: "mergeProps",
      retainedSignature: "\\/\\^\\[a-z\\]\\$\\/u",
      maximumJavaScriptGzipBytes: 700,
      maximumCssGzipBytes: 0,
    },
    aliases: ["merge props", "slot presence", "has slot content"],
    upstreamCoverage: ["React Aria mergeProps", "Vue mergeProps", "Reka UI renderSlotFragments"],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
  {
    canonicalName: "variants",
    title: "Typed Class Variants",
    packageSubpath: "./variants",
    entryFile: `${variantsFamilyRoot}variants.ts`,
    sourceFiles: [`${variantsFamilyRoot}variants.ts`, `${variantsFamilyRoot}variants-types.ts`],
    behaviorContract: `${variantsFamilyRoot}variants.behavior.md`,
    tests: [`${variantsFamilyRoot}variants.test.ts`],
    typeTests: [`${variantsFamilyRoot}variants.types.test-d.ts`],
    qualityGates: [...stableQualityGates, "ssr", "hydration"],
    bundleBudget: {
      exportName: "defineVariants",
      retainedSignature: "VIZE_UI_VARIANTS_BREAKPOINT",
      maximumJavaScriptGzipBytes: 1_200,
      maximumCssGzipBytes: 0,
    },
    aliases: ["cva", "class variance authority", "tailwind variants", "recipe", "cx", "clsx"],
    upstreamCoverage: [
      "class-variance-authority",
      "tailwind-variants",
      "Panda CSS recipes",
      "clsx",
    ],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
