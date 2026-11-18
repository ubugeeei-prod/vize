import assert from "node:assert/strict";

export type Picomatch = ((pattern: string, options?: Record<string, unknown>) => unknown) & {
  makeRe(pattern: string, options?: Record<string, unknown>): RegExp;
  isMatch(input: string, pattern: string, options?: Record<string, unknown>): boolean;
};

// GHSA-fc5q-39xc-vjpf: a single-branch repeated extglob with a wildcard star
// nests one unbounded quantifier in another. The repaired parser compiles such
// a body once (`+(X)` as `@(X)`, `*(X)` as `?(X)`), which matches the same
// strings, and treats a body mixing a star with other syntax as literal text.
export const collapsedSources: readonly (readonly [string, string])[] = [
  ["+(*)", "^(?:(?=.)(?:[^/]*?))$"],
  ["+(*a)", "^(?:(?=.)(?:[^/]*?a))$"],
  ["*(*a)", "^(?:(?=.)(?:[^/]*?a)?)$"],
  ["+(a*)", "^(?:(?=.)(?:a[^/]*?))$"],
  ["+(a*b)", "^(?:(?=.)(?:a[^/]*?b))$"],
  ["+(*.js)", "^(?:(?=.)(?:[^/]*?\\.js))$"],
  ["x/+(*a)/y", "^(?:x\\/(?:[^/]*?a)\\/y)$"],
];
export const literalSources: readonly (readonly [string, string])[] = [
  ["+(*[ab])", "^(?:\\+\\(\\*\\[ab\\]\\))$"],
  ["+(*/*)", "^(?:\\+\\(\\*\\/\\*\\))$"],
  ["+(*(a)*)", "^(?:\\+\\(\\*\\(a\\)\\*\\))$"],
];
// Pattern and non-matching input pairs that backtrack exponentially in the
// unrepaired 2.3.2 and 4.0.7 parsers: about 0.25 s at 24 characters, doubling
// with every further character.
export const pathological: readonly (readonly [string, string])[] = [
  ["+(*)", "a".repeat(64) + "/"],
  ["+(*a)", "a".repeat(64) + "b"],
  ["*(*a)", "a".repeat(64) + "b"],
  ["+(a*)", "a".repeat(64) + "/"],
  ["+(a*b)", "ab".repeat(32) + "c"],
  ["+(*[ab])", "a".repeat(64) + "c"],
  ["+(*/*)", "a/".repeat(32) + "a!"],
];
const matches: readonly (readonly [string, string, boolean])[] = [
  ["a", "+(*a)", true],
  ["ba", "+(*a)", true],
  ["abba", "+(*a)", true],
  ["ab", "+(*a)", false],
  ["b/a", "+(*a)", false],
  ["aba", "*(*a)", true],
  ["xba", "x*(*a)", true],
  ["xab", "x*(*a)", false],
  ["abab", "+(a*b)", true],
  ["axbayb", "+(a*b)", true],
  ["aba", "+(a*b)", false],
  ["a.js.js", "+(*.js)", true],
  ["a.jsx", "+(*.js)", false],
  ["x/baba/y", "x/+(*a)/y", true],
  ["x/ab/y", "x/+(*a)/y", false],
  ["fffooo", "*(*(f)*(o))", true],
  ["src/app/main.ts", "src/**/*.ts", true],
  ["src/app/main.vue", "src/**/*.{ts,tsx}", false],
  ["foo.js", "@(foo|bar).js", true],
  ["baz.js", "!(foo|bar).js", true],
];
export const budgetMs = 250;

export function verifyPicomatchExtglob(picomatch: Picomatch): {
  negativeChecks: number;
  positiveChecks: number;
} {
  let negativeChecks = 0,
    positiveChecks = 0;
  // Check the compiled shape first, so an unrepaired copy fails fast instead
  // of hanging in the timing laws below.
  for (const [pattern, source] of [...collapsedSources, ...literalSources]) {
    assert.equal(picomatch.makeRe(pattern).source, source, `nested quantifier for ${pattern}`);
    negativeChecks++;
  }
  assert.equal(picomatch.makeRe("+(*(a))").source, "^(?:(?=.)a*)$");
  assert.equal(picomatch.makeRe("+(a|aa)").source, "^(?:\\+\\(a\\|aa\\))$");
  negativeChecks += 2;
  for (const [pattern, input] of pathological) {
    const matcher = picomatch.makeRe(pattern);
    const start = process.hrtime.bigint();
    assert.equal(matcher.test(input), false);
    const elapsed = Number(process.hrtime.bigint() - start) / 1e6;
    assert.ok(elapsed < budgetMs, `${pattern} took ${elapsed.toFixed(1)} ms`);
    negativeChecks++;
  }
  for (const [input, pattern, expected] of matches) {
    assert.equal(picomatch.isMatch(input, pattern), expected, `${input} vs ${pattern}`);
    positiveChecks++;
  }
  // The documented opt-out keeps the former repeated output.
  assert.equal(
    picomatch.makeRe("+(*a)", { maxExtglobRecursion: false }).source,
    "^(?:(?=.)(?:[^/]*?a)+)$",
  );
  positiveChecks++;
  return { negativeChecks, positiveChecks };
}
