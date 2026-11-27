import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "../../catalog/family-catalog-types.ts";

const root = "src/components/chat/" as const;

/** Catalog metadata lives with the implementation it describes. */
export const chatFamilyCatalogEntry = {
  canonicalName: "chat",
  title: "Chat",
  packageSubpath: "./chat",
  entryFile: `${root}index.ts`,
  sourceFiles: [
    `${root}chat.vue`,
    `${root}chat-message.vue`,
    `${root}chat-composer.vue`,
    `${root}index.ts`,
    `${root}types.ts`,
  ],
  behaviorContract: `${root}chat.behavior.md`,
  documentation: `${root}chat.guide.md`,
  tests: [`${root}tests/behavior.test.ts`],
  typeTests: [`${root}tests/chat.types.test-d.ts`],
  rendererFixture: "components/chat/chat.vue",
  qualityGates: componentQualityGates,
  bundleBudget: {
    exportName: "Chat",
    retainedSignature: 'data-vize-ui":(?:`chat`|"chat"|\'chat\')',
    maximumJavaScriptGzipBytes: 6_000,
    maximumCssGzipBytes: 0,
  },
  aliases: ["message log", "conversation", "ai chat", "thread"],
  upstreamCoverage: ["shadcn/ui", "Vercel AI SDK chat", "Adobe Spectrum"],
  dependencies: [],
  maturity: "stable",
  owner: catalogOwner,
} as const satisfies UiFamilyCatalogEntry;
