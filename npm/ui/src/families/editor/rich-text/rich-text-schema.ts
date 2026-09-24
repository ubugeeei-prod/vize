/** Attribute values a node or mark may carry; serializable by design. */
export type RichTextAttrValue = boolean | number | string | null;

/** Attribute record of a node or mark. */
export type RichTextAttrs = { readonly [name: string]: RichTextAttrValue };

/**
 * Structural role of a node type; commands work on roles, so custom node
 * types participate in every command without special cases.
 *
 * - `textblock`: holds inline content (paragraph, heading, code block).
 * - `container`: holds blocks (blockquote, callout).
 * - `list`: holds `listItem` nodes (bullet and ordered lists).
 * - `listItem`: holds blocks inside a list.
 * - `inline`: an atomic inline leaf (image, hard break, mention chip).
 */
export type RichTextNodeRole = "container" | "inline" | "list" | "listItem" | "textblock";

/** Element produced when rendering a node or mark. */
export interface RichTextDomSpec {
  /** Lower-case tag name. */
  readonly tag: string;

  /** Attributes; `null`/`undefined` values are omitted. */
  readonly attrs?: { readonly [name: string]: string | null | undefined };
}

/** How an HTML element maps onto a node or mark while parsing (pasting). */
export interface RichTextParseRule<Attrs extends RichTextAttrs> {
  /** Lower-case tag name this rule matches. */
  readonly tag: string;

  /** Read attributes; return `null` to reject the element. @default defaults */
  attrs?(element: Element): Partial<Attrs> | null;
}

/** Declaration of one node type. `Role` and `Attrs` flow into the document types. */
export interface RichTextNodeSpec<
  Role extends RichTextNodeRole = RichTextNodeRole,
  Attrs extends RichTextAttrs = RichTextAttrs,
> {
  /** Structural role. */
  readonly role: Role;

  /** Default attributes; also the attribute type of this node. */
  readonly defaults: Attrs;

  /** Render one node. */
  render(attrs: Attrs): RichTextDomSpec;

  /** Parse rules for pasted HTML. */
  readonly parse: readonly RichTextParseRule<Attrs>[];

  /** Textblocks only: plain text without marks or inline leaves (code blocks). */
  readonly code?: boolean;
}

/** Declaration of one mark type. */
export interface RichTextMarkSpec<Attrs extends RichTextAttrs = RichTextAttrs> {
  /** Default attributes; also the attribute type of this mark. */
  readonly defaults: Attrs;

  /** Render one mark around its text. */
  render(attrs: Attrs): RichTextDomSpec;

  /** Parse rules for pasted HTML. */
  readonly parse: readonly RichTextParseRule<Attrs>[];

  /** Whether typing at the mark's end extends it. @default true */
  readonly inclusive?: boolean;
}

/** Node specs by type name. */
export type RichTextNodeSpecs = { readonly [name: string]: RichTextNodeSpec };

/** Mark specs by type name. */
export type RichTextMarkSpecs = { readonly [name: string]: RichTextMarkSpec };

/** A validated schema. */
export interface RichTextSchema<
  Nodes extends RichTextNodeSpecs = RichTextNodeSpecs,
  Marks extends RichTextMarkSpecs = RichTextMarkSpecs,
> {
  readonly nodes: Nodes;
  readonly marks: Marks;
  /** Textblock type used for new and split blocks. */
  readonly defaultBlock: string;
}

/** Name of every node type with `Role`. */
export type RichTextNodeName<
  Schema extends RichTextSchema,
  Role extends RichTextNodeRole = RichTextNodeRole,
> = {
  [Name in keyof Schema["nodes"] & string]: Schema["nodes"][Name]["role"] extends Role
    ? Name
    : never;
}[keyof Schema["nodes"] & string];

/** Name of every mark type. */
export type RichTextMarkName<Schema extends RichTextSchema> = keyof Schema["marks"] & string;

/** Attributes of one node type. */
export type RichTextNodeAttrs<
  Schema extends RichTextSchema,
  Name extends keyof Schema["nodes"],
> = Schema["nodes"][Name]["defaults"];

/** Attributes of one mark type. */
export type RichTextMarkAttrs<
  Schema extends RichTextSchema,
  Name extends keyof Schema["marks"],
> = Schema["marks"][Name]["defaults"];

const schemaDiagnostic = "VIZE_UI_RICH_TEXT_SCHEMA";
const reservedNames = new Set(["doc", "text"]);

/**
 * Validate and freeze a schema. Node and mark names, roles, and attribute
 * types stay literal, so documents and commands are typed from it.
 *
 * ```ts
 * const schema = defineRichTextSchema({
 *   nodes: { paragraph: paragraphSpec(), heading: headingSpec(), image: imageSpec() },
 *   marks: { bold: boldSpec(), link: linkSpec() },
 * });
 * ```
 */
export function defineRichTextSchema<
  const Nodes extends RichTextNodeSpecs,
  const Marks extends RichTextMarkSpecs,
>(definition: {
  readonly nodes: Nodes;
  readonly marks: Marks;
  readonly defaultBlock?: keyof Nodes & string;
}): RichTextSchema<Nodes, Marks> {
  const names = Object.keys(definition.nodes);
  for (const name of [...names, ...Object.keys(definition.marks)]) {
    if (reservedNames.has(name) || !/^[A-Za-z][\w-]*$/u.test(name)) {
      throw new TypeError(`${schemaDiagnostic}: "${name}" is not a valid node or mark name`);
    }
  }
  const textblocks = names.filter((name) => definition.nodes[name]?.role === "textblock");
  const defaultBlock = definition.defaultBlock ?? textblocks[0];
  if (defaultBlock === undefined || definition.nodes[defaultBlock]?.role !== "textblock") {
    throw new TypeError(`${schemaDiagnostic}: the default block must be a textblock node`);
  }
  const hasList = names.some((name) => definition.nodes[name]?.role === "list");
  const hasItem = names.some((name) => definition.nodes[name]?.role === "listItem");
  if (hasList !== hasItem) {
    throw new TypeError(`${schemaDiagnostic}: list and listItem nodes must be declared together`);
  }
  return Object.freeze({ nodes: definition.nodes, marks: definition.marks, defaultBlock });
}

/** Declare a custom node type with inferred role and attributes. */
export function richTextNode<
  const Role extends RichTextNodeRole,
  const Attrs extends RichTextAttrs,
>(spec: RichTextNodeSpec<Role, Attrs>): RichTextNodeSpec<Role, Attrs> {
  return Object.freeze(spec);
}

/** Declare a custom mark type with inferred attributes. */
export function richTextMark<const Attrs extends RichTextAttrs>(
  spec: RichTextMarkSpec<Attrs>,
): RichTextMarkSpec<Attrs> {
  return Object.freeze(spec);
}

const safeUrl = /^(?:https?:|mailto:|tel:|\/|\.{0,2}\/|#|[^:]*$)/iu;
const safeImageUrl = /^(?:https?:|data:image\/(?:png|gif|jpe?g|webp|avif);|\/|\.{0,2}\/|[^:]*$)/iu;

/** Whether a link target uses a safe scheme (http(s), mailto, tel, or relative). */
export function isSafeUrl(value: string): boolean {
  return safeUrl.test(value.trim());
}

/** Whether an image source uses a safe scheme (http(s), raster data URLs, or relative). */
export function isSafeImageUrl(value: string): boolean {
  return safeImageUrl.test(value.trim());
}

/** Paragraph textblock. */
export function paragraphSpec() {
  return richTextNode({
    role: "textblock",
    defaults: {},
    render: () => ({ tag: "p" }),
    parse: [{ tag: "p" }, { tag: "div" }],
  });
}

/** Heading textblock with a `level` attribute (1–6). */
export function headingSpec() {
  const levels = [1, 2, 3, 4, 5, 6] as const;
  return richTextNode<"textblock", { readonly level: number }>({
    role: "textblock",
    defaults: { level: 1 },
    render: ({ level }) => ({ tag: `h${Math.min(6, Math.max(1, Math.trunc(level)))}` }),
    parse: levels.map((level) => ({ tag: `h${level}`, attrs: () => ({ level }) })),
  });
}

/** Code block textblock (plain text, no marks). */
export function codeBlockSpec() {
  return richTextNode<"textblock", { readonly language: string | null }>({
    role: "textblock",
    code: true,
    defaults: { language: null },
    render: ({ language }) => ({
      tag: "pre",
      attrs: { "data-language": language },
    }),
    parse: [
      {
        tag: "pre",
        attrs: (element) => ({ language: element.getAttribute("data-language") }),
      },
    ],
  });
}

/** Blockquote container. */
export function blockquoteSpec() {
  return richTextNode({
    role: "container",
    defaults: {},
    render: () => ({ tag: "blockquote" }),
    parse: [{ tag: "blockquote" }],
  });
}

/** Bullet list. */
export function bulletListSpec() {
  return richTextNode({
    role: "list",
    defaults: {},
    render: () => ({ tag: "ul" }),
    parse: [{ tag: "ul" }],
  });
}

/** Ordered list with a `start` attribute. */
export function orderedListSpec() {
  return richTextNode<"list", { readonly start: number }>({
    role: "list",
    defaults: { start: 1 },
    render: ({ start }) => ({ tag: "ol", attrs: { start: start === 1 ? null : String(start) } }),
    parse: [
      {
        tag: "ol",
        attrs: (element) => {
          const start = Number(element.getAttribute("start") ?? 1);
          return { start: Number.isSafeInteger(start) ? start : 1 };
        },
      },
    ],
  });
}

/** List item. */
export function listItemSpec() {
  return richTextNode({
    role: "listItem",
    defaults: {},
    render: () => ({ tag: "li" }),
    parse: [{ tag: "li" }],
  });
}

/** Inline image with sanitized `src`. */
export function imageSpec() {
  return richTextNode<"inline", { readonly src: string; readonly alt: string }>({
    role: "inline",
    defaults: { src: "", alt: "" },
    render: ({ src, alt }) => ({
      tag: "img",
      attrs: { src: isSafeImageUrl(src) ? src : null, alt },
    }),
    parse: [
      {
        tag: "img",
        attrs: (element) => {
          const src = element.getAttribute("src") ?? "";
          return isSafeImageUrl(src) && src !== ""
            ? { src, alt: element.getAttribute("alt") ?? "" }
            : null;
        },
      },
    ],
  });
}

/** Inline hard line break. */
export function hardBreakSpec() {
  return richTextNode({
    role: "inline",
    defaults: {},
    render: () => ({ tag: "br" }),
    parse: [{ tag: "br" }],
  });
}

/** Bold mark. */
export function boldSpec() {
  return richTextMark({
    defaults: {},
    render: () => ({ tag: "strong" }),
    parse: [{ tag: "strong" }, { tag: "b" }],
  });
}

/** Italic mark. */
export function italicSpec() {
  return richTextMark({
    defaults: {},
    render: () => ({ tag: "em" }),
    parse: [{ tag: "em" }, { tag: "i" }],
  });
}

/** Underline mark. */
export function underlineSpec() {
  return richTextMark({ defaults: {}, render: () => ({ tag: "u" }), parse: [{ tag: "u" }] });
}

/** Strikethrough mark. */
export function strikeSpec() {
  return richTextMark({
    defaults: {},
    render: () => ({ tag: "s" }),
    parse: [{ tag: "s" }, { tag: "del" }, { tag: "strike" }],
  });
}

/** Inline code mark. */
export function codeSpec() {
  return richTextMark({ defaults: {}, render: () => ({ tag: "code" }), parse: [{ tag: "code" }] });
}

/** Link mark with a sanitized `href`; not inclusive, so typing after a link leaves it. */
export function linkSpec() {
  return richTextMark<{ readonly href: string }>({
    inclusive: false,
    defaults: { href: "" },
    render: ({ href }) => ({
      tag: "a",
      attrs: { href: isSafeUrl(href) ? href : null, rel: "noopener noreferrer" },
    }),
    parse: [
      {
        tag: "a",
        attrs: (element) => {
          const href = element.getAttribute("href") ?? "";
          return href !== "" && isSafeUrl(href) ? { href } : null;
        },
      },
    ],
  });
}

/** A schema with every built-in node and mark. */
export function createDefaultRichTextSchema() {
  return defineRichTextSchema({
    nodes: {
      paragraph: paragraphSpec(),
      heading: headingSpec(),
      blockquote: blockquoteSpec(),
      codeBlock: codeBlockSpec(),
      bulletList: bulletListSpec(),
      orderedList: orderedListSpec(),
      listItem: listItemSpec(),
      image: imageSpec(),
      hardBreak: hardBreakSpec(),
    },
    marks: {
      bold: boldSpec(),
      italic: italicSpec(),
      underline: underlineSpec(),
      strike: strikeSpec(),
      code: codeSpec(),
      link: linkSpec(),
    },
  });
}

/** Type of {@link createDefaultRichTextSchema}. */
export type RichTextDefaultSchema = ReturnType<typeof createDefaultRichTextSchema>;
