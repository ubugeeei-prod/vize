interface LocationRecord {
  readonly file: string;
  readonly ruleId: string;
  readonly severity: string;
  readonly line: number | bigint;
  readonly column: number | bigint;
  readonly endLine: number | bigint;
  readonly endColumn: number | bigint;
  readonly message: string;
}

interface RuleLocationRule {
  readonly reason: string;
  subject(positive: LocationRecord, negative: LocationRecord): string | null;
}

export interface RuleLocationMatch {
  readonly subject: string;
  readonly reason: string;
}

interface DestructuredProps {
  readonly open: number;
  readonly close: number;
  readonly typeName: string;
}

interface TypeBody {
  readonly open: number;
  readonly close: number;
  readonly name: string;
}

export interface RealProjectLocationSource {
  readonly source: string;
  readonly destructures: readonly DestructuredProps[];
  readonly typeBodies: readonly TypeBody[];
}

/** Rule-owned anchors shared by the current and compatibility comparators. */
export const realProjectLocationRules: Readonly<Record<string, RuleLocationRule>> = {
  "vue/require-component-is": {
    subject: () => "component-without-is",
    reason:
      "both tools report the same dynamic component without is; Patina reports the element while eslint-plugin-vue extends the range through recovered following content.",
  },
  "vue/require-component-registration": {
    subject: (positive, negative) =>
      containsRange(negative, positive)
        ? `component-tag:${positive.line}:${positive.column}`
        : null,
    reason:
      "Patina preserves the authored component tag-name anchor while eslint-plugin-vue reports the complete owning start tag.",
  },
  "vue/require-prop-comment": {
    subject(positive, negative) {
      const prop = propCommentName(positive.message);
      return prop !== undefined &&
        prop === propCommentName(negative.message) &&
        containsRange(negative, positive)
        ? `prop:${prop}`
        : null;
    },
    reason:
      "Patina reports the authored prop name in its declaration while eslint-plugin-vue reports the complete declaration or destructured binding.",
  },
  "vue/quote-props": {
    subject: (positive, negative) =>
      sameStart(positive, negative) && containsRange(negative, positive)
        ? `property:${positive.line}:${positive.column}`
        : null,
    reason:
      "Patina reports the quoted property key while eslint-plugin-vue reports the complete owning property.",
  },
};

/** Pair reviewed rule-owned anchors while retaining both complete packets. */
export function realProjectLocationDivergence(
  positive: LocationRecord,
  negative: LocationRecord,
  positives: readonly LocationRecord[] = [positive],
  negatives: readonly LocationRecord[] = [negative],
  source: RealProjectLocationSource | null = null,
): RuleLocationMatch | null {
  const rule = realProjectLocationRules[positive.ruleId];
  if (rule === undefined) return null;
  if (
    positive.ruleId === "vue/require-prop-comment" &&
    !hasUniquePropCommentPair(positive, negatives, positives)
  )
    return null;
  const subject =
    positive.ruleId === "vue/require-prop-comment" &&
    source !== null &&
    linkedDefinePropsDeclarations(source, positive, negative)
      ? `prop:${propCommentName(positive.message)}`
      : rule.subject(positive, negative);
  return subject === null
    ? null
    : {
        subject,
        reason: rule.reason,
      };
}

function hasUniquePropCommentPair(
  positive: LocationRecord,
  negatives: readonly LocationRecord[],
  positives: readonly LocationRecord[],
): boolean {
  const prop = propCommentName(positive.message);
  if (prop === undefined) return false;
  const samePacketClass = (record: LocationRecord) =>
    record.file === positive.file &&
    record.ruleId === positive.ruleId &&
    record.severity === positive.severity &&
    propCommentName(record.message) === prop;
  return (
    positives.filter(samePacketClass).length === 1 && negatives.filter(samePacketClass).length === 1
  );
}

function sameStart(left: LocationRecord, right: LocationRecord): boolean {
  return left.line === right.line && left.column === right.column;
}

function linkedDefinePropsDeclarations(
  context: RealProjectLocationSource,
  positive: LocationRecord,
  negative: LocationRecord,
): boolean {
  const { source } = context;
  const prop = propCommentName(positive.message);
  if (prop === undefined || prop !== propCommentName(negative.message)) return false;
  const positiveStart = sourceOffset(source, positive.line, positive.column),
    positiveEnd = sourceOffset(source, positive.endLine, positive.endColumn),
    negativeStart = sourceOffset(source, negative.line, negative.column);
  if (positiveStart === null || positiveEnd === null || negativeStart === null) return false;
  if (source.slice(positiveStart, positiveEnd) !== prop) return false;
  if (
    !source.startsWith(prop, negativeStart) ||
    /[\w$]/u.test(source[negativeStart + prop.length] ?? "")
  )
    return false;

  return context.destructures.some(
    (destructure) =>
      negativeStart > destructure.open &&
      negativeStart < destructure.close &&
      context.typeBodies.some(
        (body) =>
          body.name === destructure.typeName &&
          positiveStart > body.open &&
          positiveStart < body.close &&
          isDirectTypeMember(context.source, body, positiveStart),
      ),
  );
}

export function indexRealProjectLocationSource(source: string): RealProjectLocationSource {
  const destructures: DestructuredProps[] = [],
    pattern = /\b(?:const|let|var)\s*\{/gu;
  for (const match of source.matchAll(pattern)) {
    const start = match.index ?? 0,
      open = start + match[0].lastIndexOf("{"),
      close = matchingBrace(source, open);
    if (!executableOffset(source, start) || close === null) continue;
    const assignment = /^\s*=\s*defineProps\s*<\s*([A-Za-z_$][\w$]*)\s*>\s*\(/u.exec(
        source.slice(close + 1),
      ),
      typeName = assignment?.[1];
    if (typeName !== undefined) destructures.push({ open, close, typeName });
  }
  const typeBodies: TypeBody[] = [],
    declaration =
      /\b(?:type\s+([A-Za-z_$][\w$]*)\s*=|interface\s+([A-Za-z_$][\w$]*)(?:\s+extends[^{]+)?)\s*\{/gu;
  for (const match of source.matchAll(declaration)) {
    const start = match.index ?? 0,
      name = match[1] ?? match[2],
      open = start + match[0].lastIndexOf("{"),
      close = matchingBrace(source, open);
    if (!executableOffset(source, start) || name === undefined || close === null) continue;
    typeBodies.push({ open, close, name });
  }
  return { source, destructures, typeBodies };
}

function isDirectTypeMember(source: string, body: TypeBody, offset: number): boolean {
  let braces = 1,
    brackets = 0,
    parentheses = 0;
  for (let index = body.open + 1; index < offset;) {
    const character = source[index];
    if (character === '"' || character === "'" || character === "`") {
      index = quotedEnd(source, index);
    } else if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      index = newline < 0 ? source.length : newline + 1;
    } else if (source.startsWith("/*", index)) {
      const close = source.indexOf("*/", index + 2);
      index = close < 0 ? source.length : close + 2;
    } else {
      if (character === "{") braces += 1;
      else if (character === "}") braces -= 1;
      else if (character === "[") brackets += 1;
      else if (character === "]") brackets -= 1;
      else if (character === "(") parentheses += 1;
      else if (character === ")") parentheses -= 1;
      index += 1;
    }
  }
  return braces === 1 && brackets === 0 && parentheses === 0;
}

function executableOffset(source: string, offset: number): boolean {
  for (let index = 0; index < offset;) {
    const character = source[index];
    if (character === '"' || character === "'" || character === "`") {
      const end = quotedEnd(source, index);
      if (offset < end) return false;
      index = end;
    } else if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2),
        end = newline < 0 ? source.length : newline + 1;
      if (offset < end) return false;
      index = end;
    } else if (source.startsWith("/*", index)) {
      const close = source.indexOf("*/", index + 2),
        end = close < 0 ? source.length : close + 2;
      if (offset < end) return false;
      index = end;
    } else index += 1;
  }
  return true;
}

function matchingBrace(source: string, open: number): number | null {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    const character = source[index];
    if (character === '"' || character === "'" || character === "`") {
      index = quotedEnd(source, index) - 1;
    } else if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      index = newline < 0 ? source.length : newline;
    } else if (source.startsWith("/*", index)) {
      const close = source.indexOf("*/", index + 2);
      index = close < 0 ? source.length : close + 1;
    } else if (character === "{") depth += 1;
    else if (character === "}" && --depth === 0) return index;
  }
  return null;
}

function quotedEnd(source: string, open: number): number {
  const quote = source[open];
  for (let index = open + 1; index < source.length; index += 1) {
    if (source[index] === "\\") index += 1;
    else if (source[index] === quote) return index + 1;
  }
  return source.length;
}

function sourceOffset(
  source: string,
  line: number | bigint,
  column: number | bigint,
): number | null {
  const row = Number(line),
    col = Number(column);
  if (!Number.isSafeInteger(row) || !Number.isSafeInteger(col) || row < 1 || col < 1) return null;
  let offset = 0;
  for (let current = 1; current < row; current += 1) {
    const newline = source.indexOf("\n", offset);
    if (newline < 0) return null;
    offset = newline + 1;
  }
  const result = offset + col - 1;
  return result <= source.length ? result : null;
}

function containsRange(outer: LocationRecord, inner: LocationRecord): boolean {
  const startsBefore =
      outer.line < inner.line || (outer.line === inner.line && outer.column <= inner.column),
    endsAfter =
      outer.endLine > inner.endLine ||
      (outer.endLine === inner.endLine && outer.endColumn >= inner.endColumn);
  return startsBefore && endsAfter;
}

function propCommentName(message: string): string | undefined {
  return /The "([^"]+)" property should have a JSDoc comment\./u.exec(message)?.[1];
}
