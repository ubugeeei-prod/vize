// Stage feed (Levels P2-18, C-2/C-5): the compiler's stage pages as one
// schema-versioned document. The committed schema is
// `docs/davinci/plan/stage-ladder-feed.schema.json`. The stage tab reads
// `negotiateProductCapture`; this module only keeps the page shape the
// ladder still places.

/** Canonical display names. Wire stage ids stay unchanged. */
export function layerId(stage: string): string {
  if (!/^s[0-4](?:$|-)/.test(stage)) return stage;
  return stage.replace(/^s([0-4])/, "l$1").replace(/-to-s([0-4])$/, "-to-l$1");
}

export interface StagePage {
  /** Source file the page was produced for, or null for a pipeline run. */
  path: string | null;
  /** Stage (or stage page family): `s1`, `s2`, `s3`, `s3-partition`, `s3-values`. */
  stage: string;
  /** Producing step: a pass name, `lower`, or `parse`. */
  pass: string;
  /** Canonical dump text (L1: the byte-faithful surface render). */
  text: string;
}

/** One optimization remark (P3-13) for the file at `path`. */
export interface StageFeedRemark {
  path: string | null;
  stage: string;
  pass: string;
  kind: "applied" | "missed" | "analysis";
  name: string;
  /** Byte offsets in the same frame as that file's pages. */
  span: { start: number; end: number };
  args: { key: string; value: string | number | boolean }[];
}

export interface StageFeed {
  schema_version: number;
  command: string;
  pages: StagePage[];
  /** Additive to v1: absent means no remarks. */
  remarks?: StageFeedRemark[];
}
