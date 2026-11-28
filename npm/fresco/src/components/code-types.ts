import type { Ref } from "vue";
import type { FrescoAppearance } from "../protocol.js";

/** Zero-based grapheme position in a code buffer. */
export interface CodePosition {
  readonly line: number;
  readonly column: number;
}

/** Anchor and focus positions defining a grapheme-based code selection. */
export interface CodeSelection {
  readonly anchor: CodePosition;
  readonly focus: CodePosition;
}

/** Display severity of an annotation in the code viewport. */
export type CodeDiagnosticSeverity = "error" | "warning" | "info" | "hint";

/** Message and optional grapheme range attached to one logical code line. */
export interface CodeDiagnostic {
  /** Zero-based logical line in the buffer. */
  readonly line: number;
  readonly startColumn?: number;
  readonly endColumn?: number;
  readonly severity?: CodeDiagnosticSeverity;
  readonly message: string;
}

/** A literal source fragment with optional terminal appearance. */
export interface CodeSyntaxToken extends FrescoAppearance {
  readonly text: string;
}

/** Language and line coordinates supplied to a synchronous syntax provider. */
export interface CodeSyntaxContext {
  readonly language?: string;
  readonly line: number;
  readonly lineNumber: number;
}

/** Dependency-free synchronous extension point for optional syntax packages. */
export interface CodeSyntaxProvider {
  tokenize(line: string, context: CodeSyntaxContext): readonly CodeSyntaxToken[];
}

/** Reactive append-friendly text source for streamed logs and source. */
export interface CodeBuffer {
  readonly content: Readonly<Ref<string>>;
  append(chunk: string): void;
  replace(content: string): void;
  clear(): void;
}

/** Source, buffer, selection, diagnostics, syntax provider, and viewport inputs for Code. */
export interface CodeProps {
  /** Source text used when no buffer is supplied.
   * @default ""
   */
  code?: string;
  /** Streaming buffer whose current content takes precedence over code.
   * @default undefined
   */
  buffer?: CodeBuffer;
  /** Language label displayed above the code and passed to syntax providers.
   * @default undefined
   */
  language?: string;
  /** Render line-number gutters.
   * @default true
   */
  lineNumbers?: boolean;
  /** Displayed number assigned to the first source line.
   * @default 1
   */
  startLine?: number;
  /** Displayed line numbers to highlight; omission creates a fresh array per component instance.
   * @default []
   */
  highlightLines?: readonly number[];
  /** Border style; none also removes padding.
   * @default "single"
   */
  border?: "single" | "double" | "rounded" | "none";
  /** Fallback foreground color for source spans.
   * @default "white"
   */
  fg?: string;
  /** Foreground color of line-number gutters.
   * @default "gray"
   */
  lineNumberFg?: string;
  /** Background color of highlighted, unselected source spans.
   * @default "blue"
   */
  highlightBg?: string;
  /** Background color of selected source spans.
   * @default "blue"
   */
  selectionBg?: string;
  /** Diagnostics by zero-based source line; omission creates a fresh array per component instance.
   * @default []
   */
  diagnostics?: readonly CodeDiagnostic[];
  /** Optional consumer-owned syntax tokenizer.
   * @default undefined
   */
  syntaxProvider?: CodeSyntaxProvider;
  /** Controlled selection; omission retains the component internal selection.
   * @default undefined
   */
  selection?: CodeSelection;
  /** Enable keyboard selection while focused.
   * @default true
   */
  selectable?: boolean;
  /** Own keyboard selection while selectable and highlight the active gutter.
   * @default false
   */
  focused?: boolean;
  /** Requested first visible source-line index, clamped to the available viewport.
   * @default 0
   */
  offset?: number;
  /** Visible source-line count; omission displays every line.
   * @default undefined
   */
  viewportSize?: number;
  /** Extra source lines retained outside each viewport edge.
   * @default 2
   */
  overscan?: number;
}
