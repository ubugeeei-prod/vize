import type { Ref } from "vue";
import type { FrescoAppearance } from "../protocol.js";

/** Zero-based grapheme position in a code buffer. */
export interface CodePosition {
  readonly line: number;
  readonly column: number;
}

export interface CodeSelection {
  readonly anchor: CodePosition;
  readonly focus: CodePosition;
}

export type CodeDiagnosticSeverity = "error" | "warning" | "info" | "hint";

export interface CodeDiagnostic {
  /** Zero-based logical line in the buffer. */
  readonly line: number;
  readonly startColumn?: number;
  readonly endColumn?: number;
  readonly severity?: CodeDiagnosticSeverity;
  readonly message: string;
}

export interface CodeSyntaxToken extends FrescoAppearance {
  readonly text: string;
}

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

export interface CodeProps {
  code?: string;
  buffer?: CodeBuffer;
  language?: string;
  lineNumbers?: boolean;
  startLine?: number;
  highlightLines?: readonly number[];
  border?: "single" | "double" | "rounded" | "none";
  fg?: string;
  lineNumberFg?: string;
  highlightBg?: string;
  selectionBg?: string;
  diagnostics?: readonly CodeDiagnostic[];
  syntaxProvider?: CodeSyntaxProvider;
  selection?: CodeSelection;
  selectable?: boolean;
  focused?: boolean;
  offset?: number;
  viewportSize?: number;
  overscan?: number;
}
