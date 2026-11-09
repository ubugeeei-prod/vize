/**
 * TypeScript's content-mapper host reads a five-element span as `FeatureAll`
 * (`spanmap.Unmarshal`) and never reads an ignore directive's sixth element.
 * Vize writes those short forms; audits compare packets in the full form.
 */

/** `spanmap.FeatureAll`. */
export const SPAN_FEATURES_ALL = (1 << 20) - 1;
const DIRECTIVE_POLICY_IGNORE = 0;

const rows = (value: unknown): unknown[][] =>
  Array.isArray(value) ? value.filter((row): row is unknown[] => Array.isArray(row)) : [];

function widenOutput(output: unknown): void {
  if (output === null || typeof output !== "object") return;
  const record = output as Record<string, unknown>;
  for (const span of rows(record.mappings)) if (span.length === 5) span.push(SPAN_FEATURES_ALL);
  const directives = (record.diagnosticDirectives as Record<string, unknown> | undefined)
    ?.directives;
  for (const row of rows(directives)) {
    if (row.length === 5 && row[4] === DIRECTIVE_POLICY_IGNORE) row.push(0);
  }
}

/** Restore six-element tuples in a decoded JSON-RPC message, in place. */
export function sixElementTuples<T>(message: T): T {
  if (message !== null && typeof message === "object") {
    widenOutput((message as Record<string, unknown>).result);
  }
  return message;
}
