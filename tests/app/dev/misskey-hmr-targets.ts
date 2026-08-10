export interface MisskeyHmrTarget {
  expectedSha256: string;
  marker: string;
  moduleSuffix: string;
  originalAnchor: string;
  sourceRelativePath: string;
  updatedAnchor: string;
}

export const MISSKEY_HMR_EXTERNAL_TEMPLATE =
  '<template src="./MkVisitorDashboard.hmr.html"></template>';

export const MISSKEY_HMR_TARGETS = [
  {
    expectedSha256: "3427b052175dc48cf90c9ba8909ef9557e588cb4a97a8c93328147e589f26506",
    marker: "data-vize-hmr-direct",
    moduleSuffix: "/src/ui/visitor.vue.ts",
    originalAnchor: '<div :class="$style.root">',
    sourceRelativePath: "src/ui/visitor.vue",
    updatedAnchor: '<div :class="$style.root" data-vize-hmr-direct="updated">',
  },
  {
    expectedSha256: "0d0f7fe5b2623c58b18fca7ed2a99d6645a55e78e5582bd50ad14ade3e8ee545",
    marker: "data-vize-hmr-dependency",
    moduleSuffix: "/src/components/MkVisitorDashboard.vue.ts",
    originalAnchor: '<div v-if="instance" :class="$style.root">',
    sourceRelativePath: "src/components/MkVisitorDashboard.hmr.html",
    updatedAnchor: '<div v-if="instance" :class="$style.root" data-vize-hmr-dependency="updated">',
  },
] as const satisfies readonly MisskeyHmrTarget[];
