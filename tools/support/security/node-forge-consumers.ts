import assert from "node:assert/strict";

// Match literal module values, including npm aliases, URL paths and deep files.
export function protectedModule(value: string): boolean {
  try {
    value = decodeURIComponent(value);
  } catch {
    /* Keep an invalid URI literal verbatim. */
  }
  return /(?:^|[/:])(?:node-forge|listhen)(?:@[^/?#]*|\/[^?#]*)?(?:[?#].*)?$/.test(value);
}

function cooked(text: string): string {
  return text.replace(
    /\\(?:u\{([0-9a-fA-F]+)\}|u([0-9a-fA-F]{4})|x([0-9a-fA-F]{2})|([0-7]{1,3})|(\r\n|[\s\S]))/g,
    (_match, point: string, unicode: string, hex: string, octal: string, character: string) => {
      if (point || unicode || hex)
        return String.fromCodePoint(parseInt(point || unicode || hex, 16));
      if (octal) return String.fromCharCode(parseInt(octal, 8));
      if (character === "\n" || character === "\r" || character === "\r\n") return "";
      return (
        ({ n: "\n", r: "\r", t: "\t", b: "\b", f: "\f", v: "\v" } as Record<string, string>)[
          character
        ] ?? character
      );
    },
  );
}

export function assertNoPackageConsumers(source: string): void {
  const literal = /"((?:\\[\s\S]|[^"\\])*)"|'((?:\\[\s\S]|[^'\\])*)'|`((?:\\[\s\S]|[^`\\])*)`/g;
  const gap = String.raw`(?:\s|\/\*[\s\S]*?\*\/|\/\/[^\r\n]*(?:\r?\n|$))*`;
  const imports = new RegExp(
    "\\b(?:from|import|require)\\b" + gap + "(?:\\(" + gap + ")?" + literal.source,
    "g",
  );
  for (const match of source.matchAll(imports)) {
    const specifier = cooked(match[1] ?? match[2] ?? match[3]);
    assert.ok(!protectedModule(specifier), "direct Forge/listhen literal consumer: " + specifier);
  }
  for (const match of source.matchAll(literal)) {
    const value = cooked(match[1] ?? match[2] ?? match[3]);
    if (/^https?:\/\//.test(value)) assert.ok(!protectedModule(value), "new CDN consumer");
    assert.doesNotMatch(value, /(?:^|\/)node-forge(?:@[^/\s"'<>]+)?\/(?:dist|lib)\//);
  }
}
