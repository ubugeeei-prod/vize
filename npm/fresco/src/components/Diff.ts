/** Unified and side-by-side text diff with bounded viewport rendering. */

import { computed, defineComponent, h, type PropType, type VNode } from "vue";

import {
  createSplitRows,
  createUnifiedRows,
  type DiffHunkHeader,
  type DiffLine,
  type DiffSegment,
  type SplitDiffLine,
} from "./diff-algorithm.js";
import { LineNumber } from "./LineNumber.js";

export type DiffView = "unified" | "split";

export interface DiffProps {
  oldText: string;
  newText: string;
  view?: DiffView;
  context?: number;
  showLineNumbers?: boolean;
  viewportStart?: number;
  viewportSize?: number;
  overscan?: number;
  oldLabel?: string;
  newLabel?: string;
  contextFg?: string;
  addedFg?: string;
  addedBg?: string;
  removedFg?: string;
  removedBg?: string;
  hunkFg?: string;
  lineNumberFg?: string;
  addedHighlightBg?: string;
  removedHighlightBg?: string;
}

function normalizedInteger(value: number | undefined, fallback: number): number {
  if (value === undefined || !Number.isFinite(value)) return fallback;
  return Math.max(0, Math.trunc(value));
}

export const Diff = defineComponent({
  name: "Diff",
  props: {
    oldText: { type: String, required: true },
    newText: { type: String, required: true },
    view: { type: String as PropType<DiffView>, default: "unified" },
    context: { type: Number, default: 3 },
    showLineNumbers: { type: Boolean, default: true },
    viewportStart: { type: Number, default: 0 },
    viewportSize: Number,
    overscan: { type: Number, default: 2 },
    oldLabel: String,
    newLabel: String,
    contextFg: { type: String, default: "white" },
    addedFg: { type: String, default: "green" },
    addedBg: String,
    removedFg: { type: String, default: "red" },
    removedBg: String,
    hunkFg: { type: String, default: "cyan" },
    lineNumberFg: { type: String, default: "gray" },
    addedHighlightBg: { type: String, default: "darkgreen" },
    removedHighlightBg: { type: String, default: "darkred" },
  },
  setup(props) {
    const unifiedRows = computed(() =>
      createUnifiedRows(props.oldText, props.newText, props.context),
    );
    const rows = computed(() =>
      props.view === "split" ? createSplitRows(unifiedRows.value) : unifiedRows.value,
    );
    const visibleRows = computed(() => {
      if (props.viewportSize === undefined) return rows.value;
      const start = normalizedInteger(props.viewportStart, 0);
      const size = normalizedInteger(props.viewportSize, 0);
      const overscan = normalizedInteger(props.overscan, 2);
      return rows.value.slice(Math.max(0, start - overscan), start + size + overscan);
    });
    const oldLineWidth = computed(
      () => String(Math.max(1, props.oldText.split(/\r?\n/u).length)).length,
    );
    const newLineWidth = computed(
      () => String(Math.max(1, props.newText.split(/\r?\n/u).length)).length,
    );

    const blankGutter = (width: number) =>
      h("text", { text: " ".repeat(width + 3), "aria-hidden": true });
    const gutter = (line: number | undefined, width: number) => {
      if (!props.showLineNumbers) return undefined;
      return line === undefined
        ? blankGutter(width)
        : h(LineNumber, { value: line, width, fg: props.lineNumberFg });
    };
    const prefix = (line: DiffLine | undefined) => {
      const text = line?.kind === "added" ? "+ " : line?.kind === "removed" ? "- " : "  ";
      return h("text", { text, bold: line?.kind !== "context" });
    };
    const lineColors = (line: DiffLine | undefined) => {
      if (line?.kind === "added") return { fg: props.addedFg, bg: props.addedBg };
      if (line?.kind === "removed") return { fg: props.removedFg, bg: props.removedBg };
      return { fg: props.contextFg, bg: undefined };
    };
    const segmentNodes = (line: DiffLine | undefined): VNode[] => {
      if (!line) return [h("text", { text: " " })];
      const colors = lineColors(line);
      return line.segments.map((segment: DiffSegment, index) =>
        h("text", {
          key: index,
          text: segment.text || " ",
          fg: colors.fg,
          bg: segment.changed
            ? line.kind === "added"
              ? props.addedHighlightBg
              : line.kind === "removed"
                ? props.removedHighlightBg
                : colors.bg
            : colors.bg,
          underline: segment.changed && line.kind !== "context",
        }),
      );
    };
    const renderHunk = (row: DiffHunkHeader) =>
      h("text", { text: row.text, fg: props.hunkFg, bold: true, "aria-label": "Diff hunk" });
    const renderUnifiedLine = (line: DiffLine) =>
      h(
        "box",
        { style: { flexDirection: "row" }, "aria-label": `${line.kind} diff line` },
        [
          gutter(line.oldLine, oldLineWidth.value),
          gutter(line.newLine, newLineWidth.value),
          prefix(line),
          ...segmentNodes(line),
        ].filter((node): node is VNode => node !== undefined),
      );
    const renderSplitPane = (line: DiffLine | undefined, width: number, side: "old" | "new") =>
      h(
        "box",
        {
          style: { flexDirection: "row", flexGrow: 1 },
          "aria-label": `${side} diff pane`,
        },
        [
          gutter(side === "old" ? line?.oldLine : line?.newLine, width),
          prefix(line),
          ...segmentNodes(line),
        ].filter((node): node is VNode => node !== undefined),
      );
    const renderSplitLine = (line: SplitDiffLine) =>
      h("box", { style: { flexDirection: "row", gap: 1 } }, [
        renderSplitPane(line.oldLine, oldLineWidth.value, "old"),
        renderSplitPane(line.newLine, newLineWidth.value, "new"),
      ]);
    const renderLabels = () => {
      if (!props.oldLabel && !props.newLabel) return undefined;
      return h("box", { style: { flexDirection: "row", gap: 1 }, "aria-label": "Diff labels" }, [
        h("text", { text: props.oldLabel ? `--- ${props.oldLabel}` : "" }),
        h("text", { text: props.newLabel ? `+++ ${props.newLabel}` : "" }),
      ]);
    };

    return () =>
      h(
        "box",
        {
          style: { flexDirection: "column" },
          "aria-label": `${props.view} diff, ${rows.value.length} rows`,
        },
        [
          renderLabels(),
          ...visibleRows.value.map((row) =>
            row.kind === "hunk"
              ? renderHunk(row)
              : props.view === "split"
                ? renderSplitLine(row as SplitDiffLine)
                : renderUnifiedLine(row as DiffLine),
          ),
        ].filter((node): node is VNode => node !== undefined),
      );
  },
});
