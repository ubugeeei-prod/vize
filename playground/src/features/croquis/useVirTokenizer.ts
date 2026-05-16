/** Token types for VIR syntax highlighting. */
export type VirTokenType =
  | "border"
  | "section"
  | "section-name"
  | "macro"
  | "type"
  | "binding"
  | "identifier"
  | "tag"
  | "source"
  | "arrow"
  | "number"
  | "diagnostic"
  | "keyword"
  | "colon"
  | "bracket"
  | "plain";

export interface VirToken {
  type: VirTokenType;
  text: string;
}

export interface VirLine {
  tokens: VirToken[];
  index: number;
  lineType: string;
}

/** Tokenize a single VIR line into fine-grained syntax tokens. */
export function tokenizeVirLine(line: string): VirToken[] {
  const tokens: VirToken[] = [];
  let remaining = line;

  while (remaining.length > 0) {
    let matched = false;

    // Border characters: ╭╰│├└─┌┐╮╯┤┬┴┼
    const borderMatch = remaining.match(/^[╭╰│├└─┌┐╮╯┤┬┴┼]+/);
    if (borderMatch) {
      tokens.push({ type: "border", text: borderMatch[0] });
      remaining = remaining.slice(borderMatch[0].length);
      matched = true;
      continue;
    }

    // Section marker ■
    if (remaining.startsWith("■")) {
      tokens.push({ type: "section", text: "■" });
      remaining = remaining.slice(1);
      matched = true;
      continue;
    }

    // Section names (all caps words)
    const sectionNameMatch = remaining.match(
      /^(MACROS|BINDINGS|SCOPES|PROPS|EMITS|CSS|DIAGNOSTICS|STATS|SUMMARY)/,
    );
    if (sectionNameMatch) {
      tokens.push({ type: "section-name", text: sectionNameMatch[0] });
      remaining = remaining.slice(sectionNameMatch[0].length);
      matched = true;
      continue;
    }

    // Macro names @defineProps, @defineEmits, etc.
    const macroMatch = remaining.match(/^@\w+/);
    if (macroMatch) {
      tokens.push({ type: "macro", text: macroMatch[0] });
      remaining = remaining.slice(macroMatch[0].length);
      matched = true;
      continue;
    }

    // Type annotations <...>
    const typeMatch = remaining.match(/^<[^>]+>/);
    if (typeMatch) {
      tokens.push({ type: "type", text: typeMatch[0] });
      remaining = remaining.slice(typeMatch[0].length);
      matched = true;
      continue;
    }

    // Binding marker ▸
    if (remaining.startsWith("▸")) {
      tokens.push({ type: "binding", text: "▸" });
      remaining = remaining.slice(1);
      matched = true;
      continue;
    }

    // Arrow →
    if (remaining.startsWith("→")) {
      tokens.push({ type: "arrow", text: "→" });
      remaining = remaining.slice(1);
      matched = true;
      continue;
    }

    // Diagnostic icons
    const diagMatch = remaining.match(/^[✗⚠ℹ✓]/);
    if (diagMatch) {
      tokens.push({ type: "diagnostic", text: diagMatch[0] });
      remaining = remaining.slice(diagMatch[0].length);
      matched = true;
      continue;
    }

    // Tags in brackets [SetupRef], [Module], etc.
    const tagMatch = remaining.match(/^\[[A-Za-z][A-Za-z0-9_]*\]/);
    if (tagMatch) {
      tokens.push({ type: "tag", text: tagMatch[0] });
      remaining = remaining.slice(tagMatch[0].length);
      matched = true;
      continue;
    }

    // Keywords like type:, args:, scoped:, etc.
    const keywordMatch = remaining.match(
      /^(type|args|scoped|selectors|v-bind|start|end|depth|parent|bindings|children):/,
    );
    if (keywordMatch) {
      tokens.push({ type: "keyword", text: keywordMatch[1] });
      tokens.push({ type: "colon", text: ":" });
      remaining = remaining.slice(keywordMatch[0].length);
      matched = true;
      continue;
    }

    // Source types (ref, computed, props, etc.) - after keywords
    const sourceMatch = remaining.match(
      /^\b(ref|computed|reactive|props|emits|local|import|function|class|unknown)\b/,
    );
    if (sourceMatch) {
      tokens.push({ type: "source", text: sourceMatch[0] });
      remaining = remaining.slice(sourceMatch[0].length);
      matched = true;
      continue;
    }

    // Numbers including ranges like [0:100]
    const numberMatch = remaining.match(/^\d+/);
    if (numberMatch) {
      tokens.push({ type: "number", text: numberMatch[0] });
      remaining = remaining.slice(numberMatch[0].length);
      matched = true;
      continue;
    }

    // Brackets and braces
    const bracketMatch = remaining.match(/^[[\]{}()]/);
    if (bracketMatch) {
      tokens.push({ type: "bracket", text: bracketMatch[0] });
      remaining = remaining.slice(bracketMatch[0].length);
      matched = true;
      continue;
    }

    // Colons (standalone)
    if (remaining.startsWith(":")) {
      tokens.push({ type: "colon", text: ":" });
      remaining = remaining.slice(1);
      matched = true;
      continue;
    }

    // Identifiers (variable names, etc.)
    const identMatch = remaining.match(/^[a-zA-Z_][a-zA-Z0-9_]*/);
    if (identMatch) {
      tokens.push({ type: "identifier", text: identMatch[0] });
      remaining = remaining.slice(identMatch[0].length);
      matched = true;
      continue;
    }

    // Whitespace
    const wsMatch = remaining.match(/^\s+/);
    if (wsMatch) {
      tokens.push({ type: "plain", text: wsMatch[0] });
      remaining = remaining.slice(wsMatch[0].length);
      matched = true;
      continue;
    }

    // Any other character
    if (!matched) {
      tokens.push({ type: "plain", text: remaining[0] });
      remaining = remaining.slice(1);
    }
  }

  return tokens;
}

/** Determine the overall line type for row-level styling. */
export function getVirLineType(line: string): string {
  if (line.startsWith("╭") || line.startsWith("│") || line.startsWith("╰")) return "header";
  if (line.includes("■")) return "section";
  if (line.includes("@define") || line.includes("┌─ @")) return "macro";
  if (line.includes("▸")) return "binding";
  if (line.includes("├─") || line.includes("└─")) return "tree";
  if (line.includes("✗") || line.includes("⚠")) return "diagnostic";
  return "plain";
}

/**
 * Parse raw VIR text into an array of tokenized lines.
 * Each line includes its tokens, index, and overall line type classification.
 */
export function parseVirLines(virText: string): VirLine[] {
  if (!virText) return [];
  const lines = virText.split("\n");
  // Remove trailing empty line if present
  if (lines.length > 0 && lines[lines.length - 1] === "") {
    lines.pop();
  }
  return lines.map((line, index) => ({
    tokens: tokenizeVirLine(line),
    index,
    lineType: getVirLineType(line),
  }));
}
