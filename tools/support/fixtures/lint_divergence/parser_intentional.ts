interface ParserFinding {
  readonly ruleId: string;
  readonly line: number | bigint;
  readonly column: number | bigint;
  readonly message: string;
}

const nativeForeignTags = new Set([
  "circle",
  "defs",
  "g",
  "path",
  "pattern",
  "rect",
  "svg",
  "symbol",
  "use",
]);

/** Reviewed vue-eslint-parser errors that do not describe Vue runtime syntax. */
export function baselineParserIntentionalReason(
  finding: ParserFinding,
  source: string,
): string | null {
  if (finding.ruleId !== "vue/no-parsing-error") return null;
  if (finding.message === "Parsing error: duplicate-attribute.") {
    const opening = authoredOpening(finding, source);
    if (opening !== null && hasCaseDistinctEventNames(opening))
      return "vue-eslint-parser folds event attribute names as HTML and reports case-distinct native-renderer events as duplicates; Patina preserves their authored casing because the target platform can distinguish those event names.";
  }
  if (finding.message === "Parsing error: x-invalid-namespace.") {
    const opening = authoredOpening(finding, source);
    const tag = opening === null ? undefined : /^<\s*([A-Za-z][\w:.-]*)/u.exec(opening)?.[1];
    if (tag !== undefined && nativeForeignTags.has(tag) && /\sxmlns\s*=/u.test(opening))
      return "Patina recognizes this fragment-root SVG element as native and accepts its authored SVG namespace declaration; vue-eslint-parser loses the foreign-element context and reports x-invalid-namespace.";
  }
  return null;
}

function authoredOpening(finding: ParserFinding, source: string): string | null {
  const lines = source.split("\n");
  const line = Number(finding.line),
    column = Number(finding.column);
  if (!Number.isSafeInteger(line) || !Number.isSafeInteger(column) || line < 1 || column < 1)
    return null;
  let offset = column - 1;
  for (let index = 0; index < line - 1; index++) offset += (lines[index]?.length ?? 0) + 1;
  const start = source.lastIndexOf("<", offset),
    end = source.indexOf(">", offset);
  return start < 0 || end < offset ? null : source.slice(start, end + 1);
}

function hasCaseDistinctEventNames(opening: string): boolean {
  const seen = new Map<string, string>();
  for (const match of opening.matchAll(/(?:@|v-on:)([A-Za-z][\w:.-]*)/gu)) {
    const authored = match[1];
    if (authored === undefined) continue;
    const folded = authored.toLowerCase(),
      prior = seen.get(folded);
    if (prior !== undefined && prior !== authored) return true;
    seen.set(folded, authored);
  }
  return false;
}
