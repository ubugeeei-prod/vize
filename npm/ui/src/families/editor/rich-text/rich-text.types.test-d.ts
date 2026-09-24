/** Compile-only assertions for the rich-text schema, document, and command types. */

import type {
  RichTextBlock,
  RichTextCommand,
  RichTextDoc,
  RichTextInline,
  RichTextMark,
  RichTextMarkName,
  RichTextNodeName,
  RichTextState,
  RtDoc,
  RtState,
} from "./rich-text.ts";
import {
  createRichTextCommands,
  defineRichTextSchema,
  headingSpec,
  imageSpec,
  linkSpec,
  paragraphSpec,
  richTextMark,
  richTextNode,
  RichTextRoot,
  RichTextToolbarButton,
} from "./rich-text.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const schema = defineRichTextSchema({
  nodes: {
    paragraph: paragraphSpec(),
    heading: headingSpec(),
    image: imageSpec(),
    callout: richTextNode<"container", { readonly tone: "info" | "warn" }>({
      role: "container",
      defaults: { tone: "info" },
      render: ({ tone }) => ({ tag: "aside", attrs: { "data-tone": tone } }),
      parse: [{ tag: "aside" }],
    }),
  },
  marks: {
    link: linkSpec(),
    highlight: richTextMark<{ readonly color: string }>({
      defaults: { color: "yellow" },
      render: ({ color }) => ({ tag: "mark", attrs: { "data-color": color } }),
      parse: [{ tag: "mark" }],
    }),
  },
});
type Schema = typeof schema;

type _Textblocks = Expect<Equal<RichTextNodeName<Schema, "textblock">, "heading" | "paragraph">>;
type _Wrappers = Expect<Equal<RichTextNodeName<Schema, "container" | "list">, "callout">>;
type _Inline = Expect<Equal<RichTextNodeName<Schema, "inline">, "image">>;
type _Marks = Expect<Equal<RichTextMarkName<Schema>, "highlight" | "link">>;

type Block = RichTextBlock<Schema>;
type _BlockTypes = Expect<Equal<Block["type"], "callout" | "heading" | "paragraph">>;
type HeadingBlock = Extract<Block, { readonly type: "heading" }>;
type _HeadingAttrs = Expect<Equal<HeadingBlock["attrs"], { readonly level: number }>>;
type _HeadingContent = Expect<Equal<HeadingBlock["content"], readonly RichTextInline<Schema>[]>>;
type CalloutBlock = Extract<Block, { readonly type: "callout" }>;
type _CalloutContent = Expect<Equal<CalloutBlock["content"], readonly RichTextBlock<Schema>[]>>;
type _CalloutTone = Expect<Equal<CalloutBlock["attrs"]["tone"], "info" | "warn">>;
type HighlightMark = Extract<RichTextMark<Schema>, { readonly type: "highlight" }>;
type _MarkAttrs = Expect<Equal<HighlightMark["attrs"], { readonly color: string }>>;
type ImageLeaf = Extract<RichTextInline<Schema>, { readonly type: "image" }>;
type _ImageAttrs = Expect<
  Equal<ImageLeaf["attrs"], { readonly src: string; readonly alt: string }>
>;

const document: RichTextDoc<Schema> = {
  type: "doc",
  content: [
    { type: "heading", attrs: { level: 2 }, content: [{ type: "text", text: "Hi", marks: [] }] },
    {
      type: "callout",
      attrs: { tone: "warn" },
      content: [
        {
          type: "paragraph",
          attrs: {},
          content: [
            { type: "text", text: "x", marks: [{ type: "highlight", attrs: { color: "red" } }] },
            { type: "image", attrs: { src: "/a.png", alt: "" }, marks: [] },
          ],
        },
      ],
    },
  ],
};
const erased: RtDoc = document;

const commands = createRichTextCommands(schema);
const toggle: RichTextCommand = commands.toggleMark("highlight", { color: "blue" });
commands.setBlockType("heading", { level: 3 });
commands.wrapIn("callout", { tone: "info" });
commands.insertInline("image", { src: "/b.png" });
const agnostic: RichTextCommand = toggle;
declare const state: RichTextState<Schema>;
toggle(state);

type RootProps = Parameters<typeof RichTextRoot<Schema>>[0];
const rootProps: RootProps = {
  schema,
  defaultValue: document,
  "onUpdate:modelValue": (value) => value.content,
};
const buttonProps: InstanceType<typeof RichTextToolbarButton>["$props"] = {
  command: toggle,
  ariaLabel: "Mark",
};

// @ts-expect-error unknown mark names are rejected.
commands.toggleMark("bold");
// @ts-expect-error mark attributes are typed per mark.
commands.toggleMark("highlight", { color: 1 });
// @ts-expect-error setBlockType only accepts textblocks.
commands.setBlockType("callout");
// @ts-expect-error wrapIn only accepts containers and lists.
commands.wrapIn("paragraph");
// @ts-expect-error heading attrs are typed.
commands.setBlockType("heading", { level: "2" });
const badDoc: RichTextDoc<Schema> = {
  type: "doc",
  content: [
    // @ts-expect-error textblocks hold inline content, not blocks.
    { type: "paragraph", attrs: {}, content: [{ type: "paragraph", attrs: {}, content: [] }] },
  ],
};
// @ts-expect-error callout tone is a closed union.
const badTone: CalloutBlock["attrs"] = { tone: "error" };

void [agnostic, badDoc, badTone, buttonProps, erased, rootProps];
const probeState: RtState = state;
void probeState;
