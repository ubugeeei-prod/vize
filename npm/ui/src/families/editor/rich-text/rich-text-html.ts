import {
  assertRichTextDoc,
  blockContent,
  emptyBlock,
  inlineContent,
  isText,
  normalizeInline,
  roleOf,
  sameMark,
  textblockSize,
} from "./rich-text-model.ts";
import type { RichTextDoc, RtBlock, RtDoc, RtInline, RtMark } from "./rich-text-model.ts";
import type {
  RichTextAttrs,
  RichTextDomSpec,
  RichTextParseRule,
  RichTextSchema,
} from "./rich-text-schema.ts";
import { normalizeDoc } from "./rich-text-transform.ts";

/** Options for {@link richTextToHtml}. */
export interface RichTextHtmlOptions {
  /**
   * Emit editor hooks (`data-rt-path`, `data-rt-textblock`, `data-rt-leaf`,
   * caret fillers) used by `RichTextContent` for selection mapping.
   *
   * @default false
   */
  readonly editor?: boolean;
}

const htmlEscapes: Readonly<Record<string, string>> = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;",
};

/** Escape text or attribute content for HTML. */
export function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/gu, (character) => htmlEscapes[character] ?? character);
}

const voidTags = new Set(["br", "hr", "img", "input", "wbr"]);
const tagName = /^[a-z][a-z0-9-]*$/u;
const attrName = /^[a-z_:][a-z0-9_.:-]*$/u;

function openTag(spec: RichTextDomSpec, extra: Readonly<Record<string, string>> = {}): string {
  if (!tagName.test(spec.tag))
    throw new TypeError(`VIZE_UI_RICH_TEXT_HTML: invalid tag "${spec.tag}"`);
  let html = `<${spec.tag}`;
  for (const [name, value] of Object.entries({ ...spec.attrs, ...extra })) {
    // Event handler attributes are never emitted, whatever a custom spec returns.
    if (value === null || value === undefined || !attrName.test(name) || name.startsWith("on")) {
      continue;
    }
    html += ` ${name}="${escapeHtml(value)}"`;
  }
  return `${html}>`;
}

function closeTag(spec: RichTextDomSpec): string {
  return voidTags.has(spec.tag) ? "" : `</${spec.tag}>`;
}

function markOrder(schema: RichTextSchema, marks: readonly RtMark[]): RtMark[] {
  const order = Object.keys(schema.marks);
  return [...marks].sort((left, right) => order.indexOf(left.type) - order.indexOf(right.type));
}

function leafHtml(schema: RichTextSchema, node: RtInline, editor: boolean): string {
  if (isText(node)) return escapeHtml(node.text);
  const spec = schema.nodes[node.type]?.render(node.attrs) ?? { tag: "span" };
  const hooks: Record<string, string> = editor
    ? { "data-rt-leaf": "", contenteditable: "false" }
    : {};
  return `${openTag(spec, hooks)}${closeTag(spec)}`;
}

/** Serialize inline content, sharing mark elements across adjacent runs. */
function inlineHtml(schema: RichTextSchema, nodes: readonly RtInline[], editor: boolean): string {
  let html = "";
  const open: { mark: RtMark; spec: RichTextDomSpec }[] = [];
  for (const node of nodes) {
    const marks = markOrder(schema, node.marks);
    let keep = 0;
    while (keep < marks.length) {
      const current = open[keep];
      const mark = marks[keep];
      if (!current || !mark || !sameMark(current.mark, mark)) break;
      keep++;
    }
    while (open.length > keep) {
      const closing = open.pop();
      if (closing) html += closeTag(closing.spec);
    }
    for (const mark of marks.slice(keep)) {
      const spec = schema.marks[mark.type]?.render(mark.attrs);
      if (!spec) continue;
      open.push({ mark, spec });
      html += openTag(spec);
    }
    html += leafHtml(schema, node, editor);
  }
  while (open.length > 0) {
    const closing = open.pop();
    if (closing) html += closeTag(closing.spec);
  }
  return html;
}

function blockHtml(
  schema: RichTextSchema,
  block: RtBlock,
  path: readonly number[],
  editor: boolean,
): string {
  const role = roleOf(schema, block.type);
  const spec = schema.nodes[block.type]?.render(block.attrs) ?? { tag: "div" };
  const hooks: Record<string, string> = editor
    ? {
        "data-rt-path": path.join("."),
        ...(role === "textblock" ? { "data-rt-textblock": "" } : {}),
      }
    : {};
  let inner: string;
  if (role === "textblock") {
    const inline = inlineContent(block);
    inner = inlineHtml(schema, inline, editor);
    const last = inline.at(-1);
    // A trailing hard break (or an empty block) needs a filler to hold a caret line.
    if (
      editor &&
      (textblockSize(block) === 0 ||
        (last && !isText(last) && schema.nodes[last.type]?.render(last.attrs).tag === "br"))
    ) {
      inner += '<br data-rt-filler="">';
    }
  } else {
    inner = blockContent(block)
      .map((child, index) => blockHtml(schema, child, [...path, index], editor))
      .join("");
  }
  return `${openTag(spec, hooks)}${inner}${closeTag(spec)}`;
}

/** Serialize a document to HTML (escaped text, sanitized attributes). */
export function richTextToHtml(
  schema: RichTextSchema,
  doc: RtDoc,
  options: RichTextHtmlOptions = {},
): string {
  return doc.content
    .map((block, index) => blockHtml(schema, block, [index], options.editor === true))
    .join("");
}

// ---------------------------------------------------------------------------
// Parsing (paste sanitation)
// ---------------------------------------------------------------------------

/** Options for {@link richTextFromHtml}. */
export interface RichTextParseOptions {
  /**
   * Document whose `DOMParser` parses the HTML.
   *
   * @default globalThis.document
   */
  readonly document?: Document;
}

const droppedTags = new Set([
  "button",
  "embed",
  "head",
  "iframe",
  "input",
  "link",
  "meta",
  "noscript",
  "object",
  "script",
  "select",
  "style",
  "svg",
  "template",
  "textarea",
  "title",
]);

const blockTags = new Set([
  "address",
  "article",
  "aside",
  "blockquote",
  "dd",
  "div",
  "dl",
  "dt",
  "fieldset",
  "figcaption",
  "figure",
  "footer",
  "form",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "header",
  "hr",
  "li",
  "main",
  "nav",
  "ol",
  "p",
  "pre",
  "section",
  "table",
  "tbody",
  "td",
  "tfoot",
  "th",
  "thead",
  "tr",
  "ul",
]);

interface MatchedRule {
  readonly name: string;
  readonly attrs: RichTextAttrs;
}

function matchRule(
  specs: Readonly<
    Record<
      string,
      {
        readonly defaults: RichTextAttrs;
        readonly parse: readonly RichTextParseRule<RichTextAttrs>[];
      }
    >
  >,
  element: Element,
  accept: (name: string) => boolean,
): MatchedRule | null {
  const tag = element.localName;
  for (const [name, spec] of Object.entries(specs)) {
    if (!accept(name)) continue;
    for (const rule of spec.parse) {
      if (rule.tag !== tag) continue;
      const parsed = rule.attrs ? rule.attrs(element) : {};
      if (parsed === null) continue;
      const attrs: Record<string, RichTextAttrs[string]> = { ...spec.defaults };
      for (const [key, value] of Object.entries(parsed)) {
        if (value !== undefined && Object.hasOwn(spec.defaults, key)) attrs[key] = value;
      }
      return { name, attrs };
    }
  }
  return null;
}

class HtmlReader {
  readonly #schema: RichTextSchema;

  constructor(schema: RichTextSchema) {
    this.#schema = schema;
  }

  blocks(nodes: Iterable<Node>, marks: readonly RtMark[]): RtBlock[] {
    const blocks: RtBlock[] = [];
    let pending: RtInline[] = [];
    const flush = (): void => {
      const content = trimInline(normalizeInline(pending));
      pending = [];
      if (content.length > 0) blocks.push({ ...emptyBlock(this.#schema), content });
    };
    for (const node of nodes) {
      if (node.nodeType === 3) {
        pending.push(...this.text(node.textContent ?? "", marks, false));
        continue;
      }
      if (!isElement(node) || droppedTags.has(node.localName)) continue;
      const rule = matchRule(
        this.#schema.nodes,
        node,
        (name) => roleOf(this.#schema, name) !== "inline",
      );
      const transparent =
        rule !== null &&
        roleOf(this.#schema, rule.name) === "textblock" &&
        [...node.children].some((child) => blockTags.has(child.localName));
      if (rule && !transparent) {
        flush();
        blocks.push(...this.block(rule, node));
      } else if (transparent || blockTags.has(node.localName)) {
        flush();
        blocks.push(...this.blocks(node.childNodes, marks));
      } else {
        pending.push(...this.inline(node, marks, false));
      }
    }
    flush();
    return blocks;
  }

  block(rule: MatchedRule, element: Element): RtBlock[] {
    const role = roleOf(this.#schema, rule.name);
    if (role === "textblock") {
      const code = this.#schema.nodes[rule.name]?.code === true;
      const content = normalizeInline(
        [...element.childNodes].flatMap((child) => this.inlineNode(child, [], code)),
      );
      return [
        { type: rule.name, attrs: rule.attrs, content: code ? content : trimInline(content) },
      ];
    }
    if (role === "list") {
      const itemType = Object.keys(this.#schema.nodes).find(
        (name) => this.#schema.nodes[name]?.role === "listItem",
      );
      if (!itemType) return this.blocks(element.childNodes, []);
      const items = [...element.childNodes].flatMap((child): RtBlock[] => {
        if (isElement(child) && droppedTags.has(child.localName)) return [];
        const nodes = isElement(child) && child.localName === "li" ? child.childNodes : [child];
        const content = this.blocks(nodes, []);
        return content.length > 0
          ? [{ type: itemType, attrs: this.#schema.nodes[itemType]?.defaults ?? {}, content }]
          : [];
      });
      return items.length > 0 ? [{ type: rule.name, attrs: rule.attrs, content: items }] : [];
    }
    if (role === "listItem") return this.blocks(element.childNodes, []);
    const content = this.blocks(element.childNodes, []);
    return content.length > 0 ? [{ type: rule.name, attrs: rule.attrs, content }] : [];
  }

  inlineNode(node: Node, marks: readonly RtMark[], code: boolean): RtInline[] {
    if (node.nodeType === 3) return this.text(node.textContent ?? "", marks, code);
    if (!isElement(node) || droppedTags.has(node.localName)) return [];
    return this.inline(node, marks, code);
  }

  inline(element: Element, marks: readonly RtMark[], code: boolean): RtInline[] {
    if (!code) {
      const leaf = matchRule(
        this.#schema.nodes,
        element,
        (name) => roleOf(this.#schema, name) === "inline",
      );
      if (leaf) return [{ type: leaf.name, attrs: leaf.attrs, marks }];
    } else if (element.localName === "br") {
      return [{ type: "text", text: "\n", marks: [] }];
    }
    const mark = code ? null : matchRule(this.#schema.marks, element, () => true);
    const nextMarks = mark
      ? [
          ...marks.filter((candidate) => candidate.type !== mark.name),
          { type: mark.name, attrs: mark.attrs },
        ]
      : marks;
    return [...element.childNodes].flatMap((child) => this.inlineNode(child, nextMarks, code));
  }

  text(value: string, marks: readonly RtMark[], code: boolean): RtInline[] {
    const text = code ? value : value.replace(/[\t\n\r\f ]+/gu, " ");
    return text.length > 0 ? [{ type: "text", text, marks: code ? [] : marks }] : [];
  }
}

function isElement(node: Node): node is Element {
  return node.nodeType === 1;
}

function trimInline(nodes: readonly RtInline[]): RtInline[] {
  const result = [...nodes];
  const first = result[0];
  if (first && isText(first)) result[0] = { ...first, text: first.text.replace(/^ +/u, "") };
  const lastIndex = result.length - 1;
  const last = result[lastIndex];
  if (last && isText(last)) result[lastIndex] = { ...last, text: last.text.replace(/ +$/u, "") };
  return normalizeInline(result);
}

/**
 * Parse (untrusted) HTML into a document of the schema. Only elements the
 * schema declares survive; scripts, styles, frames, forms, and event
 * attributes are dropped and URLs are checked by each spec's parse rule.
 */
export function richTextFromHtml<Schema extends RichTextSchema>(
  schema: Schema,
  html: string,
  options: RichTextParseOptions = {},
): RichTextDoc<Schema> {
  const document = options.document ?? globalThis.document;
  const Parser = document?.defaultView?.DOMParser;
  if (!Parser)
    throw new Error("VIZE_UI_RICH_TEXT_HTML: parsing HTML requires a DOM (pass options.document)");
  const parsed = new Parser().parseFromString(html, "text/html");
  const blocks = new HtmlReader(schema).blocks(parsed.body.childNodes, []);
  return assertRichTextDoc(schema, normalizeDoc(schema, { type: "doc", content: blocks }));
}
