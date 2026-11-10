/** Certify the #12458 owned per-prop check against its plain predecessor. */

// The mapper owns only the name of a per-prop assignment whose value the props
// literal carries; its authored directive starts after the name and reads
// `: $AttrValue<props, key, typeof child, contract> = value satisfies contract;`.
const legacyOwnedTail =
  /^: \$AttrValue<__[\w$]+_Props_\d+, "(?:[^"\\\n]|\\.)*", typeof [^\n]+?, (__[\w$]+_\d+_prop_[\w$]+)> = ([^]*) satisfies \1;\n$/u;
const compactOwnedTail =
  /^: \$AttrValue<(__V(?:\$\d+)?)Props(\d+), "(?:[^"\\\n]|\\.)*", typeof ([\w$]+), \1Prop\2\$([\w$]+)> = ([^]*) satisfies \1Prop\2\$\4;\n$/u;
const checkName = /^[ \t]*const __vize_prop_check_\d+_[\w$]+$/u;

/**
 * The predecessor images of one owned directive: the whole plain assignment
 * line `prefix: contract = value;`. `satisfies` may have parenthesized the
 * value, so a wrapped value also yields its unwrapped image; the caller keeps
 * the one image the predecessor packet holds.
 */
export function attrValuePredecessors(linePrefix: string, generated: string): string[] {
  if (!checkName.test(linePrefix)) return [];
  const legacy = legacyOwnedTail.exec(generated);
  const compact = compactOwnedTail.exec(generated);
  if (!legacy && !compact) return [];
  const contract = legacy ? legacy[1]! : `__${compact![3]}_${compact![2]}_prop_${compact![4]}`;
  const value = legacy ? legacy[2]! : compact![5]!;
  const images = [`${linePrefix}: ${contract} = ${value};\n`];
  if (value.startsWith("(") && value.endsWith(")"))
    images.push(`${linePrefix}: ${contract} = ${value.slice(1, -1)};\n`);
  return images;
}

/** The generated line text from its start up to `start`, both UTF-8 byte offsets. */
export function linePrefixAt(source: Buffer, start: number): string {
  const lineStart = start === 0 ? 0 : source.lastIndexOf(10, start - 1) + 1;
  return source.subarray(lineStart, start).toString("utf8");
}
