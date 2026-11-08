/** Inline text fragment produced by a Markdown renderer. */
export interface MarkdownSpan {
  /** Visible terminal text. */
  text: string;
  /** Optional foreground color. */
  fg?: string;
  /** Render the fragment with strong emphasis. */
  bold?: boolean;
  /** Render the fragment with emphasis. */
  italic?: boolean;
  /** Underline the fragment, normally for links. */
  underline?: boolean;
  /** Dim secondary syntax such as quote and code markers. */
  dim?: boolean;
}

/** One terminal row emitted by a Markdown renderer. */
export interface MarkdownRenderLine {
  /** Stable key within one render result. */
  key: string;
  /** Styled fragments in display order. */
  spans: readonly MarkdownSpan[];
}

/** A parsed ATX or Setext heading. */
export interface MarkdownHeadingBlock {
  /** Heading discriminator. */
  type: "heading";
  /** One-based source line. */
  line: number;
  /** Heading depth from one through six. */
  level: number;
  /** Heading content without Markdown markers. */
  content: string;
}

/** A parsed prose paragraph. */
export interface MarkdownParagraphBlock {
  /** Paragraph discriminator. */
  type: "paragraph";
  /** One-based source line. */
  line: number;
  /** Soft-wrapped paragraph content. */
  content: string;
}

/** A parsed block quote. */
export interface MarkdownQuoteBlock {
  /** Quote discriminator. */
  type: "quote";
  /** One-based source line. */
  line: number;
  /** Quote content without leading markers. */
  content: string;
}

/** A parsed fenced or indented code block. */
export interface MarkdownCodeBlock {
  /** Code discriminator. */
  type: "code";
  /** One-based source line. */
  line: number;
  /** Optional fenced-code information string. */
  info?: string;
  /** Literal code body. */
  content: string;
}

/** One item within an ordered or unordered Markdown list. */
export interface MarkdownListItem {
  /** Visible item content without its marker. */
  content: string;
  /** Optional task-list state. */
  checked?: boolean;
}

/** A consecutive ordered or unordered list. */
export interface MarkdownListBlock {
  /** List discriminator. */
  type: "list";
  /** One-based source line. */
  line: number;
  /** Whether list markers are numeric. */
  ordered: boolean;
  /** First numeric marker for ordered lists. */
  start?: number;
  /** Items retained in source order. */
  items: readonly MarkdownListItem[];
}

/** A CommonMark thematic break. */
export interface MarkdownThematicBreakBlock {
  /** Thematic-break discriminator. */
  type: "thematic-break";
  /** One-based source line. */
  line: number;
}

/** Lightweight CommonMark block supported by Fresco. */
export type MarkdownBlock =
  | MarkdownHeadingBlock
  | MarkdownParagraphBlock
  | MarkdownQuoteBlock
  | MarkdownCodeBlock
  | MarkdownListBlock
  | MarkdownThematicBreakBlock;

/** Theme and extension inputs supplied to Markdown renderers. */
export interface MarkdownRenderOptions {
  /** Foreground color used for headings. */
  headingFg?: string;
  /** Foreground color used for links. */
  linkFg?: string;
  /** Foreground color used for quoted markers. */
  quoteFg?: string;
  /** Foreground color used for code. */
  codeFg?: string;
  /** Providers consulted before the built-in block renderer. */
  providers?: readonly MarkdownBlockProvider[];
}

/** Extension that can replace rendering for selected Markdown blocks. */
export interface MarkdownBlockProvider {
  /** Render a block, or return undefined to delegate to the next provider. */
  render(
    block: Readonly<MarkdownBlock>,
    options: Readonly<MarkdownRenderOptions>,
  ): readonly MarkdownRenderLine[] | undefined;
}

/** Complete source-to-terminal Markdown renderer. */
export interface MarkdownRenderer {
  /** Convert Markdown source into terminal rows. */
  render(source: string, options: Readonly<MarkdownRenderOptions>): readonly MarkdownRenderLine[];
}

/** Public props for the Markdown SFC. */
export interface MarkdownProps extends MarkdownRenderOptions {
  /** CommonMark source to render. */
  source: string;
  /** Optional complete renderer replacement. */
  renderer?: MarkdownRenderer;
}
