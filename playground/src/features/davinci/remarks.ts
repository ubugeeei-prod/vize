// Optimization remarks: a pass saying why it did or did not transform
// something (LLVM `-Rpass`/`-Rpass-missed` in shape). The compiler's remark
// channel (`vize_davinci::pass::observer::remark`) carries `{pass, message,
// applied}` today but nothing emits remarks into the feed yet, so the view
// renders a typed, explicit empty state rather than inventing any.

import type { Range } from "./offsets";

export interface SpolveroRemark {
  /** The pass that emitted the remark. */
  pass: string;
  /** What it did or did not do, in the pass's words. */
  message: string;
  /** Whether the pass applied the transformation it remarks on. */
  applied: boolean;
  /** Authored template bytes the remark is about, when it names a site. */
  span: Range | null;
}

export interface RemarkSummary {
  applied: number;
  missed: number;
}

/** Counts per outcome, for the tab badge and the panel header. */
export function summarizeRemarks(remarks: readonly SpolveroRemark[]): RemarkSummary {
  let applied = 0;
  for (const remark of remarks) if (remark.applied) applied += 1;
  return { applied, missed: remarks.length - applied };
}
