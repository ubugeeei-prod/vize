/** Score from 0 (very weak) to 4 (very strong). */
export type PasswordStrengthScore = 0 | 1 | 2 | 3 | 4;

/** Stable label for each score. */
export type PasswordStrengthLabel = "very-weak" | "weak" | "fair" | "strong" | "very-strong";

/** Character-class and length checks behind the default estimate. */
export interface PasswordStrengthChecks {
  /** At least `minLength` characters. */
  readonly length: boolean;

  /** Contains a lowercase letter. */
  readonly lowercase: boolean;

  /** Contains an uppercase letter. */
  readonly uppercase: boolean;

  /** Contains a digit. */
  readonly digit: boolean;

  /** Contains a symbol or whitespace. */
  readonly symbol: boolean;
}

/** Result of {@link estimatePasswordStrength}. */
export interface PasswordStrengthEstimate {
  /** Score from 0 to 4. */
  readonly score: PasswordStrengthScore;

  /** Label for the score, suitable for `data-strength` styling. */
  readonly label: PasswordStrengthLabel;

  /** Individual checks, useful for requirement checklists. */
  readonly checks: PasswordStrengthChecks;
}

/** Options for {@link estimatePasswordStrength}. */
export interface PasswordStrengthOptions {
  /**
   * Minimum length counted as meeting the length requirement.
   *
   * @default 8
   */
  readonly minLength?: number;
}

const labels = [
  "very-weak",
  "weak",
  "fair",
  "strong",
  "very-strong",
] as const satisfies readonly PasswordStrengthLabel[];

function toScore(points: number): PasswordStrengthScore {
  if (points <= 0) return 0;
  if (points === 1) return 1;
  if (points === 2) return 2;
  return points === 3 ? 3 : 4;
}

/**
 * Dependency-free heuristic strength estimate.
 *
 * It rewards length and character variety and caps short passwords. Pass a
 * stronger estimator (for example zxcvbn) to `PasswordField` through
 * `evaluateStrength` when policy requires dictionary checks.
 */
export function estimatePasswordStrength(
  value: string,
  options: PasswordStrengthOptions = {},
): PasswordStrengthEstimate {
  const minLength =
    typeof options.minLength === "number" &&
    Number.isFinite(options.minLength) &&
    options.minLength > 0
      ? Math.floor(options.minLength)
      : 8;
  const checks: PasswordStrengthChecks = {
    length: value.length >= minLength,
    lowercase: /\p{Ll}/u.test(value),
    uppercase: /\p{Lu}/u.test(value),
    digit: /\p{Nd}/u.test(value),
    symbol: /[^\p{L}\p{Nd}]/u.test(value),
  };
  if (value.length === 0) return { score: 0, label: labels[0], checks };
  const variety = [checks.lowercase, checks.uppercase, checks.digit, checks.symbol].filter(
    Boolean,
  ).length;
  let points = variety - 1;
  if (checks.length) points += 1;
  if (value.length >= minLength * 2) points += 1;
  // Short passwords never rate above "weak", whatever their variety.
  if (!checks.length) points = Math.min(points, 1);
  const score = toScore(points);
  return { score, label: labels[score], checks };
}
