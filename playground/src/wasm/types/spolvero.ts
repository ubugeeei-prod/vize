// Spolvero feed (Davinci P2-18, C-2/C-5): the compiler's stage pages as one
// schema-versioned document. The committed schema is
// `davinci-road/plan/spolvero-feed.schema.json`; this module is the
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

export interface SpolveroFeed {
  schema_version: number;
  command: string;
  pages: SpolveroPage[];
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
  return {
    ok: true,
    feed: { schema_version: version, command: raw.command, pages: raw.pages as SpolveroPage[] },
  };
}
