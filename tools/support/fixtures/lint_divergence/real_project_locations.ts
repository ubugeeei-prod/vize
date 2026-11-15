interface LocationRecord {
  readonly ruleId: string;
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
      return prop !== undefined && prop === propCommentName(negative.message)
        ? `prop:${prop}`
        : null;
    },
    reason:
      "Patina reports the authored prop name in its declaration while eslint-plugin-vue reports the complete declaration or destructured binding.",
  },
  "vue/quote-props": {
    subject: (positive, negative) =>
      containsRange(negative, positive) ? `property:${positive.line}:${positive.column}` : null,
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
): RuleLocationMatch | null {
  const rule = realProjectLocationRules[positive.ruleId];
  if (rule === undefined) return null;
  if (
    positive.ruleId === "vue/require-prop-comment" &&
    !hasUniquePropCommentPair(positive, negatives, positives)
  )
    return null;
  const subject = rule.subject(positive, negative);
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
    record.ruleId === positive.ruleId && propCommentName(record.message) === prop;
  return (
    positives.filter(samePacketClass).length === 1 && negatives.filter(samePacketClass).length === 1
  );
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
