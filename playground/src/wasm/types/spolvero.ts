// Spolvero feed (Davinci P2-18, C-2/C-5): the compiler's stage pages as one
// schema-versioned document. The committed schema is
// `docs/davinci/plan/spolvero-feed.schema.json`; this module is the
// playground's consumer side of it and negotiates the version before reading
// any shape-dependent field (devtool.md: refuse a mismatch loudly instead of
// misrendering).

/** The feed format version this playground renders. */
export const SPOLVERO_FEED_SCHEMA_VERSION = 1;

export interface SpolveroPage {
  /** Source file the page was produced for, or null for a pipeline run. */
  path: string | null;
  /** Stage (or stage page family): `s1`, `s2`, `s3`, `s3-partition`, `s3-values`. */
  stage: string;
  /** Producing step: a pass name, `lower`, or `parse`. */
  pass: string;
  /** Canonical folio text (S1: the byte-faithful surface render). */
  text: string;
}

/** One optimization remark (P3-13) for the file at `path`. */
export interface SpolveroFeedRemark {
  path: string | null;
  stage: string;
  pass: string;
  kind: "applied" | "missed" | "analysis";
  name: string;
  /** Byte offsets in the same frame as that file's pages. */
  span: { start: number; end: number };
  args: { key: string; value: string | number | boolean }[];
}

export interface SpolveroFeed {
  schema_version: number;
  command: string;
  pages: SpolveroPage[];
  /** Additive to v1: absent means no remarks. */
  remarks?: SpolveroFeedRemark[];
}

export type SpolveroNegotiation = { ok: true; feed: SpolveroFeed } | { ok: false; error: string };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isPage(value: unknown): value is SpolveroPage {
  return (
    isRecord(value) &&
    (value.path === null || typeof value.path === "string") &&
    typeof value.stage === "string" &&
    typeof value.pass === "string" &&
    typeof value.text === "string"
  );
}

const REMARK_KINDS = new Set(["applied", "missed", "analysis"]);

function isRemark(value: unknown): value is SpolveroFeedRemark {
  return (
    isRecord(value) &&
    (value.path === null || typeof value.path === "string") &&
    typeof value.stage === "string" &&
    typeof value.pass === "string" &&
    typeof value.kind === "string" &&
    REMARK_KINDS.has(value.kind) &&
    typeof value.name === "string" &&
    isRecord(value.span) &&
    typeof value.span.start === "number" &&
    typeof value.span.end === "number" &&
    Array.isArray(value.args) &&
    value.args.every(
      (arg) =>
        isRecord(arg) &&
        typeof arg.key === "string" &&
        ["string", "number", "boolean"].includes(typeof arg.value),
    )
  );
}

/**
 * Accept a raw `spolvero` member only when its `schema_version` is the one
 * this view understands and every page has the committed shape.
 */
export function negotiateSpolveroFeed(raw: unknown): SpolveroNegotiation {
  if (!isRecord(raw)) {
    return { ok: false, error: "The compiler result carries no Spolvero feed." };
  }
  const version = raw.schema_version;
  if (typeof version !== "number") {
    return { ok: false, error: "The Spolvero feed has no numeric schema_version." };
  }
  if (version !== SPOLVERO_FEED_SCHEMA_VERSION) {
    return {
      ok: false,
      error: `Spolvero feed schema_version ${version} is not supported; this view renders version ${SPOLVERO_FEED_SCHEMA_VERSION}.`,
    };
  }
  if (typeof raw.command !== "string" || !Array.isArray(raw.pages)) {
    return { ok: false, error: "The Spolvero feed is missing its command or pages." };
  }
  const invalid = raw.pages.findIndex((page) => !isPage(page));
  if (invalid !== -1) {
    return { ok: false, error: `Spolvero feed page ${invalid} does not match the schema.` };
  }
  const remarks = raw.remarks;
  if (remarks !== undefined) {
    const bad = Array.isArray(remarks) ? remarks.findIndex((remark) => !isRemark(remark)) : 0;
    if (bad !== -1) {
      return { ok: false, error: `Spolvero feed remark ${bad} does not match the schema.` };
    }
  }
  const feed: SpolveroFeed = {
    schema_version: version,
    command: raw.command,
    pages: raw.pages as SpolveroPage[],
  };
  if (remarks !== undefined) feed.remarks = remarks as SpolveroFeedRemark[];
  return { ok: true, feed };
}
