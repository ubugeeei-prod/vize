import type { MarquetteDiagnostic } from "./validate.js";
import type { TestRunRetainedEvidence } from "./test-run-model.js";

/** Largest integer every consuming language can represent exactly. */
export const MAX_SAFE_EVIDENCE_INTEGER = 9007199254740991;

const IDENTIFIER = /^[a-z0-9][a-z0-9._-]*$/;
const DIGEST = /^[a-f0-9]{64}$/;
const SOURCE_REVISION = /^[a-f0-9]{40,128}$/;
const TIMESTAMP =
  /^([0-9]{4})-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])T([01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]\.[0-9]{3}Z$/;

/** Builds the stable diagnostic path for a named collection member. */
export function evidencePath(collection: string, id: string): string {
  return `${collection}.${id}`;
}

/** Creates one error diagnostic. */
export function error(
  code: MarquetteDiagnostic["code"],
  path: string,
  message: string,
): MarquetteDiagnostic {
  return { code, severity: "error", path, message };
}

/** Validates the shared identifier grammar and the schema length bound. */
export function checkIdentifier(
  id: string,
  path: string,
  diagnostics: MarquetteDiagnostic[],
): void {
  if (!IDENTIFIER.test(id)) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_103",
        path,
        "identifier must use lowercase ASCII letters, digits, dash, underscore, or dot",
      ),
    );
  }
  if (id.length === 0 || id.length > 128) {
    diagnostics.push(
      error("VIZE_MARQUETTE_103", path, "identifier must be between 1 and 128 characters"),
    );
  }
}

/** Validates a lowercase 64-character SHA-256 fingerprint. */
export function checkDigest(value: string, path: string, diagnostics: MarquetteDiagnostic[]): void {
  if (!DIGEST.test(value)) {
    diagnostics.push(
      error("VIZE_MARQUETTE_104", path, "fingerprint must be 64 lowercase hexadecimal characters"),
    );
  }
}

/** Validates an exact source revision digest. */
export function checkSourceRevision(
  value: string,
  path: string,
  diagnostics: MarquetteDiagnostic[],
): void {
  if (!SOURCE_REVISION.test(value)) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_105",
        path,
        "source revision must be 40 to 128 lowercase hexadecimal characters",
      ),
    );
  }
}

/** Returns whether `value` is a millisecond-precision UTC calendar instant. */
export function isStrictTimestamp(value: string): boolean {
  const match = TIMESTAMP.exec(value);
  if (match === null) {
    return false;
  }
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const maxDay =
    month === 2
      ? year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0)
        ? 29
        : 28
      : month === 4 || month === 6 || month === 9 || month === 11
        ? 30
        : 31;
  return day <= maxDay;
}

/** Validates a millisecond-precision UTC timestamp. */
export function checkTimestamp(
  value: string,
  path: string,
  diagnostics: MarquetteDiagnostic[],
): void {
  if (!isStrictTimestamp(value)) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_107",
        path,
        "timestamp must be a millisecond-precision UTC instant like 2026-01-01T00:00:00.000Z",
      ),
    );
  }
}

/** Rejects integers that lose precision in a consuming language. */
export function checkSafeInteger(
  value: number,
  path: string,
  diagnostics: MarquetteDiagnostic[],
): void {
  if (value > MAX_SAFE_EVIDENCE_INTEGER) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_111",
        path,
        "value must not exceed the largest exactly-representable integer",
      ),
    );
  }
}

/**
 * Validates an immutable retained-evidence binding.
 *
 * The retrieval reference must be content-addressed and name exactly the
 * fingerprinted bytes; anything else would let retained evidence mutate
 * behind a stable-looking record.
 */
export function checkRetainedEvidence(
  retained: TestRunRetainedEvidence,
  path: string,
  diagnostics: MarquetteDiagnostic[],
): void {
  checkDigest(retained.fingerprint, evidencePath(path, "fingerprint"), diagnostics);
  const suffix = retained.reference.startsWith("sha256:")
    ? retained.reference.slice("sha256:".length)
    : undefined;
  if (suffix === undefined || !DIGEST.test(suffix)) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_108",
        evidencePath(path, "reference"),
        "evidence reference must be sha256: followed by 64 lowercase hexadecimal characters",
      ),
    );
  } else if (suffix !== retained.fingerprint) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_109",
        evidencePath(path, "reference"),
        "content-addressed reference must name the fingerprinted content",
      ),
    );
  }
}
