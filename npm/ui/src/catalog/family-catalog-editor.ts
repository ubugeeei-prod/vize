import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const richTextRoot = "src/families/editor/rich-text/";

/** Editing surfaces. */
export const editorFamilyCatalog = [
  {
    canonicalName: "rich-text",
    title: "Rich Text",
    packageSubpath: "./rich-text",
    entryFile: `${richTextRoot}rich-text.ts`,
    sourceFiles: [
      `${richTextRoot}rich-text.ts`,
      `${richTextRoot}rich-text-bubble-menu.vue`,
      `${richTextRoot}rich-text-commands.ts`,
      `${richTextRoot}rich-text-content.vue`,
      `${richTextRoot}rich-text-context.ts`,
      `${richTextRoot}rich-text-dom.ts`,
      `${richTextRoot}rich-text-html.ts`,
      `${richTextRoot}rich-text-input-rules.ts`,
      `${richTextRoot}rich-text-keymap.ts`,
      `${richTextRoot}rich-text-model.ts`,
      `${richTextRoot}rich-text-root.vue`,
      `${richTextRoot}rich-text-schema.ts`,
      `${richTextRoot}rich-text-state.ts`,
      `${richTextRoot}rich-text-toolbar-button.vue`,
      `${richTextRoot}rich-text-toolbar-runtime.ts`,
      `${richTextRoot}rich-text-toolbar.vue`,
      `${richTextRoot}rich-text-transform.ts`,
      `${richTextRoot}rich-text-types.ts`,
    ],
    behaviorContract: `${richTextRoot}rich-text.behavior.md`,
    tests: [
      `${richTextRoot}rich-text.test.ts`,
      `${richTextRoot}rich-text-html.test.ts`,
      `${richTextRoot}rich-text-model.test.ts`,
      `${richTextRoot}rich-text-ssr.test.ts`,
    ],
    typeTests: [`${richTextRoot}rich-text.types.test-d.ts`],
    rendererFixture: "RichTextConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "RichTextRoot",
      retainedSignature: "data-vize-ui[\"'`:=\\s]{1,4}rich-text[\"'`]",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 9_900,
      maximumCssGzipBytes: 0,
    },
    aliases: ["wysiwyg", "rich text editor", "contenteditable editor", "prose editor"],
    upstreamCoverage: ["ProseMirror model", "Tiptap", "Lexical", "Slate", "WAI-ARIA textbox"],
    dependencies: ["collection", "context", "id", "positioner"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
