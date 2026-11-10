/**
 * Projection families for the check-cost attribution: which emitter construct
 * a position in Vize's generated TS belongs to, and which helper declaration
 * encloses a position in the shared `vize/*.d.ts` files.
 *
 * Positions are UTF-8 byte offsets (tsgo trace `pos`) or 1-based
 * line/character pairs (types.json `firstDeclaration`, taken at the node's
 * full start, so leading trivia is skipped before classifying).
 */

/** Generated text with its line table; offsets are bytes. */
export type SourceText = { bytes: Buffer; lineStarts: number[] };

export function sourceText(bytes: Buffer): SourceText {
  const lineStarts = [0];
  for (let index = 0; index < bytes.length; index++) {
    if (bytes[index] === 0x0a) lineStarts.push(index + 1);
  }
  return { bytes, lineStarts };
}

/** 0-based line holding byte `offset`. */
export function lineOf(text: SourceText, offset: number): number {
  let low = 0;
  let high = text.lineStarts.length - 1;
  while (low < high) {
    const mid = (low + high + 1) >> 1;
    if ((text.lineStarts[mid] as number) <= offset) low = mid;
    else high = mid - 1;
  }
  return low;
}

export function lineText(text: SourceText, line: number): string {
  const start = text.lineStarts[line] ?? text.bytes.length;
  const end = text.lineStarts[line + 1] ?? text.bytes.length;
  return text.bytes
    .subarray(start, end)
    .toString("utf8")
    .replace(/\r?\n$/u, "");
}

/** Byte offset of a 1-based line and UTF-16 character, as types.json reports them. */
export function offsetOf(text: SourceText, line: number, character: number): number {
  const start = text.lineStarts[line - 1] ?? text.bytes.length;
  const prefix = lineText(text, line - 1).slice(0, Math.max(0, character - 1));
  return start + Buffer.byteLength(prefix, "utf8");
}

/** First non-trivia byte at or after `offset` (whitespace and comments). */
export function skipTrivia(text: SourceText, offset: number): number {
  const { bytes } = text;
  let index = offset;
  while (index < bytes.length) {
    const byte = bytes[index] as number;
    if (byte === 0x20 || byte === 0x09 || byte === 0x0a || byte === 0x0d) index++;
    else if (byte === 0x2f && bytes[index + 1] === 0x2f) {
      while (index < bytes.length && bytes[index] !== 0x0a) index++;
    } else if (byte === 0x2f && bytes[index + 1] === 0x2a) {
      const close = bytes.indexOf("*/", index + 2);
      index = close < 0 ? bytes.length : close + 2;
    } else break;
  }
  return index;
}

/** Sections of one generated SFC, by the generator's banner comments. */
const SECTIONS: Array<[section: string, marker: RegExp]> = [
  ["module", /^\/\/ =+ Module Scope/u],
  ["exported-types", /^\/\/ =+ Exported Types/u],
  ["setup-glue", /^\s*\/\/ =+ Setup Scope/u],
  ["setup-user", /^\s*\/\/ User setup code$/u],
  // The user code's own source map closes it.
  ["setup-glue", /^ {2}\/\/ @vize-map: \d+:\d+ -> \d+:\d+$/u],
  ["template", /^\s*\/\/ =+ Template Scope/u],
  ["setup-glue", /^\s*\/\/ Compiler-macro results contribute/u],
  ["component-surface", /^\/\/ Invoke setup to verify types$/u],
  ["component-export", /^\/\/ =+ Default Export/u],
];

/** Template-scope families, first match wins; each names the emitter lane that owns it. */
const TEMPLATE_FAMILIES: Array<[family: string, pattern: RegExp]> = [
  [
    "slots",
    /\$InferredSlot|__vize_slot_|\$SlotContract|__vize_required_slots|_slot_contract\b|__vizeSlotOutlet|\$StructuralSlots|\$RequiredSlots|\$SlotPayload/u,
  ],
  [
    "listener",
    /__vize_handler_|__vize_args|__vize_event\b|__vize_emit_props_|_emit_resolver\b|\$ListenerArgs|\$EmitPropsFactory|^\s*type __\w+?_\d+_\w*_(prop_args|fallthrough_args|inferred_emit_args|static_emit_args|emit_args|args|event|return_native_fallthrough|return|prop_arrays|listener|handler)\b|^\s*type __\w+?_\d+__\w+/u,
  ],
  ["events-resolve", /__vize_events_|__vize_event_keys_|__vize_kebab_events/u],
  ["prop-check", /__vize_prop_check_/u],
  ["props-call", /as unknown as __(\w+?_Check_|V(?:\$\d+)?Check)\d+\)\(/u],
  [
    "contract-alias",
    /^\s*type __\w+?_(Props|Component|CheckProps|CheckTail|ValueProps|FallthroughValue|Check)_\d+\b|^\s*type __V(?:\$\d+)?(Props|Comp|CheckProps|CheckTail|ValueProps|Check)\d+\b|^\s*type __\w+?_\d+_prop_|^\s*type __V(?:\$\d+)?Prop\d+\$|\$ComponentCheckTail|VueTemplatePublicAttrsType|\$Vue(VNode|AllowedComponent|ComponentCustom)Props/u,
  ],
  [
    "component-nav",
    /\$ComponentNavigationProps|__vize_props_nav_|^\s*const \{[^}]*\} = undefined as unknown as/u,
  ],
  ["native-element", /__vize_native_prop_check_|\$NativeElement/u],
  ["model", /_model_|__vize_model|__V(?:\$\d+)?Model\d/u],
  [
    "template-context",
    /__Ctx\b|__Global\b|\$GlobalContext|__ctx\b|\$attrs|\$slots|\$refs|\$emit\b|\$style|= props\["|\btype __U\d*</u,
  ],
  ["guard", /^\s*(\} else )?if \(/u],
];

/** Section-level families outside the template scope. */
const SECTION_FAMILIES: Record<string, Array<[family: string, pattern: RegExp]>> = {
  module: [["global-components", /\$GlobalComponent/u]],
  "component-export": [
    ["component-constructor", /\$ComponentConstructor|\$ComponentInstanceFor|\$ComponentInput\b/u],
    [
      "component-instance",
      /\$ComponentInstance\b|\$ComponentPublicBase|\$PublicEmit|\$PublicSlots/u,
    ],
  ],
};

export type SiteFamily = { section: string; family: string };

/** Section banner lines, found once per text. */
export function sectionStarts(text: SourceText): Array<[line: number, section: string]> {
  const starts: Array<[number, string]> = [];
  for (let line = 0; line < text.lineStarts.length; line++) {
    const first = text.bytes[text.lineStarts[line] as number];
    // Banners are comments; skip every other line without decoding it.
    if (first !== 0x2f && first !== 0x20) continue;
    const content = lineText(text, line);
    const match = SECTIONS.find(([, marker]) => marker.test(content));
    if (match) starts.push([line, match[0]]);
  }
  return starts;
}

/** Family of the statement at byte `offset` of a generated SFC. */
export function siteFamily(
  text: SourceText,
  sections: Array<[number, string]>,
  offset: number,
): SiteFamily {
  const start = skipTrivia(text, offset);
  const line = lineOf(text, start);
  let section = "preamble";
  for (const [from, name] of sections) {
    if (from > line) break;
    section = name;
  }
  const content = lineText(text, line);
  const rules = section === "template" ? TEMPLATE_FAMILIES : (SECTION_FAMILIES[section] ?? []);
  const rule = rules.find(([, pattern]) => pattern.test(content));
  return {
    section,
    family: rule ? rule[0] : section === "template" ? "template-expression" : section,
  };
}

const ALIAS_HEAD = /^(?:export\s+)?(?:declare\s+)?type\s+([\w$]+)\s*(<)?/u;

/** The local type alias declared at a generated position, if the statement is one. */
export function localAlias(
  text: SourceText,
  offset: number,
): { name: string; generic: boolean } | null {
  const start = skipTrivia(text, offset);
  const head = text.bytes
    .subarray(start, Math.min(text.bytes.length, start + 200))
    .toString("utf8");
  const match = ALIAS_HEAD.exec(head);
  return match ? { name: match[1] as string, generic: match[2] === "<" } : null;
}

const LISTENER_SUFFIX =
  "prop_args|fallthrough_args|inferred_emit_args|static_emit_args|emit_args|args|event|return_native_fallthrough|return|prop_arrays|listener|handler";

/** Compact per-usage contract roles, keyed by the legacy role they replace. */
const COMPACT_CONTRACT_ROLES: Record<string, string> = {
  Comp: "Component",
  Props: "Props",
  ValueProps: "ValueProps",
  CheckProps: "CheckProps",
  CheckTail: "CheckTail",
  Check: "Check",
};

/**
 * Per-usage alias names folded to their emitter shape: `__MkA_22_click_args` → `__C_N_click_args`.
 * Compact `__V<Role><N>` spellings fold to the same keys as the legacy
 * `__<Component>_<Role>_<N>` spellings, so base and head attributions compare.
 */
export function normalizeAlias(name: string): string {
  return name
    .replace(
      /^__V(?:\$\d+)?(Props|Comp|CheckProps|CheckTail|ValueProps|Check)\d+$/u,
      (_, role: string) => `__C_${COMPACT_CONTRACT_ROLES[role]}_N`,
    )
    .replace(/^__V(?:\$\d+)?Prop\d+\$[\w$]+$/u, "__C_N_prop_*")
    .replace(/^__V(?:\$\d+)?Model\d+\$[\w$]+$/u, "__C_N_model_*")
    .replace(
      /^__\w+?_(Props|Component|CheckProps|CheckTail|ValueProps|FallthroughValue|Check)_\d+$/u,
      "__C_$1_N",
    )
    .replace(/^__\w+?_\d+__(\w+)$/u, "__C_N__$1")
    .replace(/^__\w+?_\d+_prop_\w+$/u, "__C_N_prop_*")
    .replace(/^__\w+?_\d+_emit_resolver$/u, "__C_N_emit_resolver")
    .replace(new RegExp(`^__\\w+?_\\d+_[\\w$]+?_(${LISTENER_SUFFIX})$`, "u"), "__C_N_*_$1")
    .replace(/^__\w+?_slot_contract$/u, "__C_slot_contract")
    .replace(/^__\w+?_\d+_model_\w+$/u, "__C_N_model_*")
    .replace(/\d+/gu, "N");
}

const DECLARATION =
  /^\s*(?:export\s+)?(?:declare\s+)?(?:type|interface|const|let|var|function|class|namespace|enum)\s+([\w$]+)/u;

/** Top-level and block-nested declarations of a helper file: [0-based line, name], ascending. */
export function helperDeclarations(text: SourceText): Array<[line: number, name: string]> {
  const declarations: Array<[number, string]> = [];
  for (let line = 0; line < text.lineStarts.length; line++) {
    const match = DECLARATION.exec(lineText(text, line));
    if (match) declarations.push([line, match[1] as string]);
  }
  return declarations;
}

/** Name of the helper declaration enclosing a 0-based line. */
export function enclosingDeclaration(
  declarations: Array<[number, string]>,
  line: number,
): string | null {
  let name: string | null = null;
  for (const [from, declared] of declarations) {
    if (from > line) break;
    name = declared;
  }
  return name;
}
