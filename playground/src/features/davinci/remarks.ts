// Optimization remarks (P3-13): a pass saying why it did or did not
// transform something - LLVM's `-Rpass`/`-Rpass-missed`/`-Rpass-analysis`
// with structured arguments. They reach the view as the Spolvero feed's
// `remarks` member (`davinci-road/plan/remarks-format.md`); nothing here
// decides anything - every entry is a remark a pass emitted through the
// observer channel.

import type { Range } from "./offsets";

export type RemarkKind = "applied" | "missed" | "analysis";

export interface RemarkArg {
  key: string;
  value: string | number | boolean;
}

export interface SpolveroRemark {
  /** The emitting pipeline's stage (`s2`). */
  stage: string;
  /** The emitting pass, as the pass manager attributed it (`hoist-static`). */
  pass: string;
  kind: RemarkKind;
  /** The remark's name in its pass's vocabulary (`static-subtree`). */
  name: string;
  /** Authored template bytes the decision is about. */
  span: Range;
  /** Structured arguments, in the pass's order. */
  args: RemarkArg[];
}

export type RemarkSummary = Record<RemarkKind, number>;

/** Counts per kind, for the tab badge and the panel header. */
export function summarizeRemarks(remarks: readonly SpolveroRemark[]): RemarkSummary {
  const summary: RemarkSummary = { applied: 0, missed: 0, analysis: 0 };
  for (const remark of remarks) summary[remark.kind] += 1;
  return summary;
}

/** `key=value` for display, text values unquoted. */
export function formatArg(arg: RemarkArg): string {
  return `${arg.key}=${String(arg.value)}`;
}

/** Remarks about exactly the construct at `span` (an op's own span). */
export function remarksAt(remarks: readonly SpolveroRemark[], span: Range): SpolveroRemark[] {
  return remarks.filter(
    (remark) => remark.span.start === span.start && remark.span.end === span.end,
  );
}
