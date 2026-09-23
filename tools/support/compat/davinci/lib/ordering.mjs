// Deterministic code-unit ordering shared by the generator's modules. The
// artifact is byte-compared by the staleness check, so every sort passes an
// explicit comparator rather than relying on the implicit `Array#sort` one.

export function byKey(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
