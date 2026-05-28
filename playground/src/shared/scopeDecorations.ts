import * as monaco from "monaco-editor";

export interface ScopeDecorationInfo {
  start: number;
  end: number;
  kind: string;
  kindStr?: string;
  depth?: number;
  bindings?: string[];
}

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

interface ScopeHoverCopy {
  title: string;
  description: string;
  notes: string[];
}

const SCOPE_HOVER_COPY: Record<string, ScopeHoverCopy> = {
  setup: {
    title: "Script setup scope",
    description: "Top-level bindings declared in `<script setup>` and exposed to the template.",
    notes: [
      "Refs are available directly in templates.",
      "Top-level imports and constants are part of this scope.",
    ],
  },
  plain: {
    title: "Classic script scope",
    description: "Bindings from a regular `<script>` block.",
    notes: [
      "Useful for module-level code and explicit component options.",
      "Template exposure depends on returned options and setup behavior.",
    ],
  },
  extern: {
    title: "External module scope",
    description: "Symbols resolved from imported modules.",
    notes: ["Imported bindings are read-only from the template analysis point of view."],
  },
  extmod: {
    title: "External module scope",
    description: "Symbols resolved from imported modules.",
    notes: ["Imported bindings are read-only from the template analysis point of view."],
  },
  vue: {
    title: "Vue template global scope",
    description: "Built-in Vue template globals such as `$slots`, `$attrs`, and `$emit`.",
    notes: ["These symbols are provided by Vue at render time."],
  },
  universal: {
    title: "Universal runtime scope",
    description: "Globals that are safe to reference in both client and server contexts.",
    notes: ["Prefer this scope for code that must behave consistently during SSR and hydration."],
  },
  server: {
    title: "Server runtime scope",
    description: "Bindings that are only valid while rendering on the server.",
    notes: ["Keep browser-only APIs out of this scope."],
  },
  client: {
    title: "Client runtime scope",
    description: "Bindings that are only valid in the browser runtime.",
    notes: ["Use this for DOM, window, and browser lifecycle dependent code."],
  },
  vfor: {
    title: "v-for iteration scope",
    description: "Loop aliases and index bindings introduced by `v-for`.",
    notes: ["Aliases shadow outer bindings inside the repeated region."],
  },
  "v-for": {
    title: "v-for iteration scope",
    description: "Loop aliases and index bindings introduced by `v-for`.",
    notes: ["Aliases shadow outer bindings inside the repeated region."],
  },
  vslot: {
    title: "v-slot scope",
    description: "Slot props introduced by `v-slot` or shorthand slot syntax.",
    notes: ["Slot props are local to the slot template region."],
  },
  "v-slot": {
    title: "v-slot scope",
    description: "Slot props introduced by `v-slot` or shorthand slot syntax.",
    notes: ["Slot props are local to the slot template region."],
  },
  function: {
    title: "Function scope",
    description: "Bindings introduced by a function body or function parameters.",
    notes: ["Local bindings shadow parent scopes until the function exits."],
  },
  arrowfunction: {
    title: "Arrow function scope",
    description: "Bindings introduced by an arrow function body or parameters.",
    notes: ["Useful for computed callbacks, watchers, and inline handlers."],
  },
  block: {
    title: "Block scope",
    description: "Bindings introduced by a lexical block.",
    notes: ["`let` and `const` bindings stay inside this block."],
  },
  closure: {
    title: "Closure scope",
    description: "A nested function scope that captures values from parent scopes.",
    notes: ["Captured values remain visible while the closure is alive."],
  },
  event: {
    title: "Event handler scope",
    description: "Temporary scope for inline event handler expressions.",
    notes: ["Handler locals and `$event` are resolved here before parent scopes."],
  },
  callback: {
    title: "Callback scope",
    description: "Scope created for callback parameters and callback-local bindings.",
    notes: ["Common in watchers, computed callbacks, and array helpers."],
  },
};

function getScopeHoverCopy(kind: string): ScopeHoverCopy {
  const kindLower = kind.toLowerCase();
  const exact = SCOPE_HOVER_COPY[kindLower];
  if (exact) return exact;
  if (kindLower.includes("clientonly") || kindLower.includes("mounted"))
    return SCOPE_HOVER_COPY.client;
  if (kindLower.includes("server")) return SCOPE_HOVER_COPY.server;
  if (kindLower.includes("computed") || kindLower.includes("watch"))
    return SCOPE_HOVER_COPY.callback;
  return {
    title: "Semantic scope",
    description: "Region produced by Croquis semantic scope analysis.",
    notes: ["Use this region to understand which bindings are visible at the cursor."],
  };
}

function inlineCode(value: string): string {
  return `\`${value.replace(/`/g, "\\`")}\``;
}

export function getScopeDecorationHoverMessage(scope: ScopeDecorationInfo): monaco.IMarkdownString {
  const kindLabel = scope.kindStr || scope.kind;
  const copy = getScopeHoverCopy(scope.kind || kindLabel);
  const bindings = scope.bindings ?? [];
  const metadata = [
    `kind: ${kindLabel}`,
    `range: ${scope.start}-${scope.end}`,
    scope.depth == null ? null : `depth: ${scope.depth}`,
    `bindings: ${bindings.length}`,
  ]
    .filter(Boolean)
    .join("\n");

  const sections = [
    `**${copy.title}**`,
    `_Croquis scope analysis_`,
    copy.description,
    `\`\`\`text\n${metadata}\n\`\`\``,
  ];

  if (bindings.length > 0) {
    const visibleBindings = bindings.slice(0, 6).map(inlineCode).join(", ");
    const suffix = bindings.length > 6 ? `, +${bindings.length - 6} more` : "";
    sections.push(`**Bindings**\n\n${visibleBindings}${suffix}`);
  }

  sections.push(`**Note**\n- ${copy.notes[0]}`);

  return { value: sections.join("\n\n") };
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
