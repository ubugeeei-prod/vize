import * as monaco from "monaco-editor";

// Scope kind to CSS class mapping (O(1) lookup for exact matches)
const SCOPE_CLASS_MAP: Record<string, string> = {
  setup: "scope-decoration-setup",
  plain: "scope-decoration-plain",
  extern: "scope-decoration-extern",
  extmod: "scope-decoration-extern",
  vue: "scope-decoration-vue",
  universal: "scope-decoration-universal",
  server: "scope-decoration-server",
  client: "scope-decoration-client",
  vfor: "scope-decoration-vFor",
  "v-for": "scope-decoration-vFor",
  vslot: "scope-decoration-vSlot",
  "v-slot": "scope-decoration-vSlot",
  function: "scope-decoration-function",
  arrowfunction: "scope-decoration-function",
  block: "scope-decoration-block",
  mod: "scope-decoration-mod",
  closure: "scope-decoration-closure",
  event: "scope-decoration-event",
  callback: "scope-decoration-callback",
};

export function getScopeDecorationClass(kind: string): string {
  const kindLower = kind.toLowerCase();
  const exact = SCOPE_CLASS_MAP[kindLower];
  if (exact) return exact;
  if (kindLower.includes("clientonly") || kindLower.includes("mounted"))
    return "scope-decoration-client";
  if (kindLower.includes("computed")) return "scope-decoration-computed";
  if (kindLower.includes("watch")) return "scope-decoration-watch";
  return "scope-decoration-default";
}

export function offsetToPosition(
  model: monaco.editor.ITextModel,
  offset: number,
): monaco.IPosition {
  const content = model.getValue();
  const safeOffset = Math.min(offset, content.length);
  let line = 1;
  let column = 1;

  for (let i = 0; i < safeOffset; i++) {
    if (content[i] === "\n") {
      line++;
      column = 1;
    } else {
      column++;
    }
  }

  return { lineNumber: line, column };
}

// Overview ruler color mapping (O(1) lookup)
const RULER_COLOR_MAP: Record<string, string> = {
  setup: "#22c55e40",
  vue: "#42b88340",
  client: "#f97316a0",
  server: "#3b82f6a0",
  universal: "#8b5cf640",
  vfor: "#a78bfa40",
  "v-for": "#a78bfa40",
  vslot: "#f472b640",
  "v-slot": "#f472b640",
  closure: "#fbbf2440",
  block: "#94a3b830",
  event: "#f472b640",
  callback: "#fb923c40",
};

export function getOverviewRulerColor(kind: string): string {
  const kindLower = kind.toLowerCase();
  return RULER_COLOR_MAP[kindLower] || "#9ca3b020";
}
