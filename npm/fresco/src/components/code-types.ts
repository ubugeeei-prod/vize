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
