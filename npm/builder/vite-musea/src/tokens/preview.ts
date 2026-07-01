import type { DesignToken } from "./parser.js";

export type MuseaTokenPreviewKind =
  | "color"
  | "spacing"
  | "fontSize"
  | "fontWeight"
  | "lineHeight"
  | "letterSpacing"
  | "shadow"
  | "radius"
  | "opacity"
  | "zIndex"
  | "generic";

export interface MuseaTokenPreviewRule {
  kind: MuseaTokenPreviewKind;
  type?: string | string[];
  pathIncludes?: string | string[];
  nameIncludes?: string | string[];
  referencePathIncludes?: string | string[];
}

export interface MuseaTokenPreviewConfig {
  /**
   * User rules are evaluated before Musea's built-in token preview heuristics.
   * Use `kind: "generic"` to intentionally suppress a preview for matching tokens.
   */
  rules?: MuseaTokenPreviewRule[];
  /** Disable selected preview kinds after custom and built-in rules are evaluated. */
  disabledKinds?: MuseaTokenPreviewKind[];
}

export interface ResolvedTokenPreview {
  kind: MuseaTokenPreviewKind;
  value: string | number;
  tokenPath: string;
  reference?: {
    path: string;
    token?: DesignToken;
    value?: string | number;
  };
}

export interface ResolveTokenPreviewInput {
  tokenPath: string;
  token: DesignToken;
  tokenMap?: Record<string, DesignToken>;
  config?: MuseaTokenPreviewConfig;
}

interface PreviewCandidate {
  tokenPath: string;
  name: string;
  token: DesignToken;
}

export function resolveTokenPreview(input: ResolveTokenPreviewInput): ResolvedTokenPreview {
  const reference = resolveReference(input.token, input.tokenMap);
  const value = input.token.$resolvedValue ?? reference?.value ?? input.token.value;
  const candidates = buildCandidates(input.tokenPath, input.token, reference);
  const customKind = resolveCustomKind(candidates, reference?.path, input.config?.rules);
  const kind = customKind ?? inferPreviewKind(candidates, value);
  const disabledKinds = new Set(input.config?.disabledKinds ?? []);

  return {
    kind: disabledKinds.has(kind) ? "generic" : kind,
    value,
    tokenPath: input.tokenPath,
    reference,
  };
}

function resolveReference(
  token: DesignToken,
  tokenMap: Record<string, DesignToken> | undefined,
): ResolvedTokenPreview["reference"] | undefined {
  if (!token.$reference) return undefined;

  const referencedToken = tokenMap?.[token.$reference];
  return {
    path: token.$reference,
    token: referencedToken,
    value: referencedToken ? (referencedToken.$resolvedValue ?? referencedToken.value) : undefined,
  };
}

function buildCandidates(
  tokenPath: string,
  token: DesignToken,
  reference: ResolvedTokenPreview["reference"],
): PreviewCandidate[] {
  const candidates: PreviewCandidate[] = [{ tokenPath, name: pathName(tokenPath), token }];
  if (reference?.token) {
    candidates.push({
      tokenPath: reference.path,
      name: pathName(reference.path),
      token: reference.token,
    });
  }
  return candidates;
}

function resolveCustomKind(
  candidates: PreviewCandidate[],
  referencePath: string | undefined,
  rules: MuseaTokenPreviewRule[] | undefined,
): MuseaTokenPreviewKind | undefined {
  for (const rule of rules ?? []) {
    if (matchesRule(rule, candidates, referencePath)) {
      return rule.kind;
    }
  }
  return undefined;
}

function matchesRule(
  rule: MuseaTokenPreviewRule,
  candidates: PreviewCandidate[],
  referencePath: string | undefined,
): boolean {
  const checks: boolean[] = [];
  if (rule.type !== undefined)
    checks.push(candidates.some((candidate) => typeMatches(candidate, rule.type)));
  if (rule.pathIncludes !== undefined) {
    checks.push(
      candidates.some((candidate) => includesAny(candidate.tokenPath, rule.pathIncludes)),
    );
  }
  if (rule.nameIncludes !== undefined) {
    checks.push(candidates.some((candidate) => includesAny(candidate.name, rule.nameIncludes)));
  }
  if (rule.referencePathIncludes !== undefined) {
    checks.push(
      referencePath !== undefined && includesAny(referencePath, rule.referencePathIncludes),
    );
  }

  return checks.length > 0 && checks.every(Boolean);
}

function inferPreviewKind(
  candidates: PreviewCandidate[],
  value: string | number,
): MuseaTokenPreviewKind {
  if (hasType(candidates, ["color"]) || isColorValue(value)) return "color";
  if (
    hasType(candidates, ["shadow", "box-shadow"]) ||
    hasSignal(candidates, ["shadow", "box-shadow"])
  ) {
    return "shadow";
  }
  if (
    hasType(candidates, ["z-index", "zindex", "zIndex"]) ||
    hasSignal(candidates, ["z-index", "zindex", "stacking-order"])
  ) {
    return "zIndex";
  }
  if (hasType(candidates, ["opacity"]) || hasSignal(candidates, ["opacity", "alpha"])) {
    return "opacity";
  }
  if (
    hasType(candidates, ["border-radius", "borderradius", "radius"]) ||
    hasSignal(candidates, ["border-radius", "borderradius", "radius", "round", "rounded"])
  ) {
    return "radius";
  }
  if (
    hasType(candidates, ["letterspacing", "letter-spacing"]) ||
    hasSignal(candidates, ["letter-spacing", "letterspacing", "tracking"])
  ) {
    return "letterSpacing";
  }
  if (
    hasType(candidates, ["spacing", "space"]) ||
    hasSignal(candidates, ["spacing", "spasing", "space", "gap", "padding", "margin", "inset"])
  ) {
    return "spacing";
  }
  if (hasSignal(candidates, ["font-size", "fontsize", "text-size"])) return "fontSize";
  if (
    hasType(candidates, ["fontweight", "font-weight"]) ||
    hasSignal(candidates, ["font-weight", "fontweight", "weight"])
  ) {
    return "fontWeight";
  }
  if (
    hasType(candidates, ["lineheight", "line-height"]) ||
    hasSignal(candidates, ["line-height", "lineheight"])
  ) {
    return "lineHeight";
  }
  return "generic";
}

function hasType(candidates: PreviewCandidate[], terms: string[]): boolean {
  return candidates.some((candidate) => typeMatches(candidate, terms));
}

function hasSignal(candidates: PreviewCandidate[], terms: string[]): boolean {
  return candidates.some(
    (candidate) =>
      includesAny(candidate.tokenPath, terms) ||
      includesAny(candidate.name, terms) ||
      (candidate.token.type !== undefined && includesAny(candidate.token.type, terms)),
  );
}

function typeMatches(candidate: PreviewCandidate, expected: string | string[]): boolean {
  if (!candidate.token.type) return false;
  return toArray(expected).some((term) => normalize(candidate.token.type) === normalize(term));
}

function includesAny(value: string, expected: string | string[]): boolean {
  const normalizedValue = normalize(value);
  const compactValue = compact(normalizedValue);
  return toArray(expected).some((term) => {
    const normalizedTerm = normalize(term);
    return (
      normalizedValue.includes(normalizedTerm) || compactValue.includes(compact(normalizedTerm))
    );
  });
}

function isColorValue(value: string | number): boolean {
  if (typeof value !== "string") return false;
  return /^(#|rgb\(|rgba\(|hsl\(|hsla\(|oklch\(|oklab\(|lab\(|lch\(|color\()/i.test(value.trim());
}

function pathName(tokenPath: string): string {
  return tokenPath.split(".").at(-1) ?? tokenPath;
}

function normalize(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[_\s]+/g, "-");
}

function compact(value: string): string {
  return value.replace(/[-.]/g, "");
}

function toArray<T>(value: T | T[]): T[] {
  return Array.isArray(value) ? value : [value];
}
