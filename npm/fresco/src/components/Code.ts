/** Bounded code viewer with streaming, selection, diagnostics, and syntax providers. */

import { computed, defineComponent, h, ref, type PropType, watch } from "vue";
import { useInput } from "../composables/useInput.js";
import { graphemes } from "../utils/text.js";
import { LineNumber } from "./LineNumber.js";
import type {
  CodeBuffer,
  CodeDiagnostic,
  CodePosition,
  CodeProps,
  CodeSelection,
  CodeSyntaxToken,
} from "./code-types.js";

export { createCodeBuffer } from "./code-buffer.js";
export type {
  CodeBuffer,
  CodeDiagnostic,
  CodeDiagnosticSeverity,
  CodePosition,
  CodeProps,
  CodeSelection,
  CodeSyntaxContext,
  CodeSyntaxProvider,
  CodeSyntaxToken,
} from "./code-types.js";

function finiteInteger(value: number, fallback = 0): number {
  return Number.isFinite(value) ? Math.trunc(value) : fallback;
}

function comparePositions(left: CodePosition, right: CodePosition): number {
  return left.line === right.line ? left.column - right.column : left.line - right.line;
}

function position(line: number, column: number, lines: readonly string[]): CodePosition {
  const clampedLine = Math.min(Math.max(0, finiteInteger(line)), Math.max(0, lines.length - 1));
  return {
    line: clampedLine,
    column: Math.min(
      Math.max(0, finiteInteger(column)),
      graphemes(lines[clampedLine] ?? "").length,
    ),
  };
}

function normalizedSelection(selection: CodeSelection, lines: readonly string[]): CodeSelection {
  return {
    anchor: position(selection.anchor.line, selection.anchor.column, lines),
    focus: position(selection.focus.line, selection.focus.column, lines),
  };
}

function orderedSelection(selection: CodeSelection): readonly [CodePosition, CodePosition] {
  return comparePositions(selection.anchor, selection.focus) <= 0
    ? [selection.anchor, selection.focus]
    : [selection.focus, selection.anchor];
}

function selectedColumns(
  selection: CodeSelection,
  line: number,
  lineLength: number,
): readonly [number, number] | undefined {
  const [start, end] = orderedSelection(selection);
  if (comparePositions(start, end) === 0 || line < start.line || line > end.line) return undefined;
  const from = line === start.line ? start.column : 0;
  const to = line === end.line ? end.column : lineLength;
  return from < to ? [from, to] : undefined;
}

function tokensForLine(
  line: string,
  lineIndex: number,
  lineNumber: number,
  props: Readonly<CodeProps>,
): readonly CodeSyntaxToken[] {
  const tokens = props.syntaxProvider?.tokenize(line, {
    language: props.language,
    line: lineIndex,
    lineNumber,
  });
  if (!tokens || tokens.map(({ text }) => text).join("") !== line) return [{ text: line }];
  return tokens;
}

interface RenderedCodeSpan extends CodeSyntaxToken {
  readonly selected?: boolean;
  readonly diagnostic?: boolean;
}

function diagnosticAtColumn(diagnostics: readonly CodeDiagnostic[], column: number): boolean {
  return diagnostics.some((diagnostic) => {
    const start = Math.max(0, finiteInteger(diagnostic.startColumn ?? 0));
    const end = Math.max(
      start,
      finiteInteger(diagnostic.endColumn ?? Number.MAX_SAFE_INTEGER, Number.MAX_SAFE_INTEGER),
    );
    return column >= start && column < end;
  });
}

function renderedSpans(
  tokens: readonly CodeSyntaxToken[],
  selection: CodeSelection,
  line: number,
  diagnostics: readonly CodeDiagnostic[],
): RenderedCodeSpan[] {
  const text = tokens.map(({ text: value }) => value).join("");
  const selected = selectedColumns(selection, line, graphemes(text).length);
  const spans: RenderedCodeSpan[] = [];
  let column = 0;
  for (const token of tokens) {
    const parts = graphemes(token.text);
    let start = 0;
    while (start < parts.length) {
      const absolute = column + start;
      const isSelected =
        selected !== undefined && absolute >= selected[0] && absolute < selected[1];
      const hasDiagnostic = diagnosticAtColumn(diagnostics, absolute);
      let end = start + 1;
      while (end < parts.length) {
        const nextAbsolute = column + end;
        const nextSelected =
          selected !== undefined && nextAbsolute >= selected[0] && nextAbsolute < selected[1];
        if (
          nextSelected !== isSelected ||
          diagnosticAtColumn(diagnostics, nextAbsolute) !== hasDiagnostic
        ) {
          break;
        }
        end += 1;
      }
      spans.push({
        ...token,
        text: parts.slice(start, end).join(""),
        selected: isSelected,
        diagnostic: hasDiagnostic,
      });
      start = end;
    }
    column += parts.length;
  }
  return spans.length > 0 ? spans : [{ text: " " }];
}

export const Code = defineComponent({
  name: "Code",
  props: {
    code: { type: String, default: "" },
    buffer: Object as PropType<CodeBuffer>,
    language: String,
    lineNumbers: { type: Boolean, default: true },
    startLine: { type: Number, default: 1 },
    highlightLines: { type: Array as PropType<readonly number[]>, default: () => [] },
    border: { type: String as PropType<CodeProps["border"]>, default: "single" },
    fg: { type: String, default: "white" },
    lineNumberFg: { type: String, default: "gray" },
    highlightBg: { type: String, default: "blue" },
    selectionBg: { type: String, default: "blue" },
    diagnostics: { type: Array as PropType<readonly CodeDiagnostic[]>, default: () => [] },
    syntaxProvider: Object as PropType<CodeProps["syntaxProvider"]>,
    selection: Object as PropType<CodeSelection>,
    selectable: { type: Boolean, default: true },
    focused: { type: Boolean, default: false },
    offset: { type: Number, default: 0 },
    viewportSize: Number,
    overscan: { type: Number, default: 2 },
  },
  emits: ["update:selection", "update:offset", "select"],
  setup(props, { emit }) {
    const source = computed(() => props.buffer?.content.value ?? props.code);
    const lines = computed(() => source.value.split("\n"));
    const internalSelection = ref<CodeSelection>({
      anchor: { line: 0, column: 0 },
      focus: { line: 0, column: 0 },
    });
    watch(
      [() => props.selection, lines],
      ([requested]) => {
        internalSelection.value = normalizedSelection(
          requested ?? internalSelection.value,
          lines.value,
        );
      },
      { immediate: true },
    );

    const viewport = computed(() => {
      const count = lines.value.length;
      const size = Math.min(
        count,
        props.viewportSize === undefined ? count : Math.max(0, finiteInteger(props.viewportSize)),
      );
      const visibleStart = Math.min(
        Math.max(0, count - size),
        Math.max(0, finiteInteger(props.offset)),
      );
      const visibleEnd = Math.min(count, visibleStart + size);
      const overscan = Math.max(0, finiteInteger(props.overscan));
      return {
        visibleStart,
        visibleEnd,
        retainedStart: Math.max(0, visibleStart - overscan),
        retainedEnd: Math.min(count, visibleEnd + overscan),
      };
    });

    const publishSelection = (next: CodeSelection) => {
      const normalized = normalizedSelection(next, lines.value);
      internalSelection.value = normalized;
      emit("update:selection", normalized);
      emit("select", normalized);
      const size = props.viewportSize;
      if (size === undefined || size <= 0) return;
      if (normalized.focus.line < props.offset) emit("update:offset", normalized.focus.line);
      else if (normalized.focus.line >= props.offset + size) {
        emit("update:offset", normalized.focus.line - size + 1);
      }
    };

    const move = (key: string, extend: boolean) => {
      const current = internalSelection.value.focus;
      let next: CodePosition;
      const currentParts = graphemes(lines.value[current.line] ?? "");
      if (key === "left") {
        next =
          current.column > 0
            ? { line: current.line, column: current.column - 1 }
            : current.line > 0
              ? {
                  line: current.line - 1,
                  column: graphemes(lines.value[current.line - 1] ?? "").length,
                }
              : current;
      } else if (key === "right") {
        next =
          current.column < currentParts.length
            ? { line: current.line, column: current.column + 1 }
            : current.line < lines.value.length - 1
              ? { line: current.line + 1, column: 0 }
              : current;
      } else if (key === "up" || key === "down") {
        const delta = key === "up" ? -1 : 1;
        next = position(current.line + delta, current.column, lines.value);
      } else if (key === "home") next = { line: current.line, column: 0 };
      else if (key === "end") next = { line: current.line, column: currentParts.length };
      else if (key === "pageup" || key === "pagedown") {
        const delta = Math.max(1, (props.viewportSize ?? 2) - 1) * (key === "pageup" ? -1 : 1);
        next = position(current.line + delta, current.column, lines.value);
      } else return;
      publishSelection({
        anchor: extend ? internalSelection.value.anchor : next,
        focus: next,
      });
    };

    useInput({
      isActive: computed(() => props.focused && props.selectable),
      onKey: (key, modifiers) => {
        if ((modifiers.ctrl || modifiers.meta) && key.toLowerCase() === "a") {
          const lastLine = Math.max(0, lines.value.length - 1);
          publishSelection({
            anchor: { line: 0, column: 0 },
            focus: { line: lastLine, column: graphemes(lines.value[lastLine] ?? "").length },
          });
          return;
        }
        move(key.toLowerCase(), modifiers.shift);
      },
      onEscape: () => {
        publishSelection({
          anchor: internalSelection.value.focus,
          focus: internalSelection.value.focus,
        });
      },
    });

    return () => {
      const maxLineNumber = props.startLine + lines.value.length - 1;
      const lineNumberWidth = String(maxLineNumber).length;
      const children = [];
      if (props.language) {
        children.push(
          h(
            "text",
            { key: "language", dim: true, style: { marginBottom: 1 } },
            `// ${props.language}`,
          ),
        );
      }

      for (
        let lineIndex = viewport.value.retainedStart;
        lineIndex < viewport.value.retainedEnd;
        lineIndex += 1
      ) {
        const line = lines.value[lineIndex] ?? "";
        const lineNumber = props.startLine + lineIndex;
        const isVisible =
          lineIndex >= viewport.value.visibleStart && lineIndex < viewport.value.visibleEnd;
        const lineDiagnostics = props.diagnostics.filter(
          ({ line: diagnosticLine }) => diagnosticLine === lineIndex,
        );
        const parts = [];
        if (props.lineNumbers) {
          parts.push(
            h(LineNumber, {
              key: `line-number-${lineNumber}`,
              value: lineNumber,
              width: lineNumberWidth,
              fg: props.lineNumberFg,
              active: props.focused && internalSelection.value.focus.line === lineIndex,
            }),
          );
        }
        const spans = renderedSpans(
          tokensForLine(line, lineIndex, lineNumber, props),
          internalSelection.value,
          lineIndex,
          lineDiagnostics,
        );
        spans.forEach((span, spanIndex) => {
          parts.push(
            h(
              "text",
              {
                key: `code-${lineNumber}-${spanIndex}`,
                fg: span.fg ?? props.fg,
                bg: span.selected
                  ? props.selectionBg
                  : props.highlightLines.includes(lineNumber)
                    ? props.highlightBg
                    : span.bg,
                bold: span.bold,
                dim: span.dim,
                italic: span.italic,
                underline: span.underline || span.diagnostic,
              },
              span.text,
            ),
          );
        });
        if (lineDiagnostics.length > 0) {
          parts.push(
            h(
              "text",
              { key: `diagnostic-${lineNumber}`, fg: "red" },
              ` ! ${lineDiagnostics.map(({ message }) => message).join("; ")}`,
            ),
          );
        }
        children.push(
          h(
            "box",
            {
              key: `line-${lineNumber}`,
              style: { display: isVisible ? "flex" : "none", flexDirection: "row" },
              "aria-hidden": !isVisible,
              "aria-label":
                lineDiagnostics.length > 0
                  ? `Line ${lineNumber}: ${lineDiagnostics.map(({ message }) => message).join("; ")}`
                  : undefined,
              "data-code-line": lineIndex,
            },
            parts,
          ),
        );
      }

      return h(
        "box",
        {
          border: props.border === "none" ? undefined : props.border,
          style: {
            flexDirection: "column",
            overflow: "hidden",
            padding: props.border !== "none" ? 1 : 0,
          },
          "aria-label": props.language ? `${props.language} code` : "Code",
          "aria-state": { readonly: true, multiselectable: props.selectable },
        },
        children,
      );
    };
  },
});
