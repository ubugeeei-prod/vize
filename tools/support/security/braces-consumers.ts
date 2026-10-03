import assert from "node:assert/strict";

// This admits the reviewed installed Node graph only, not external module copies.
export function protectedBracesModule(value: string): boolean {
  try {
    value = decodeURIComponent(value);
  } catch {
    /* Retain invalid URI literals. */
  }
  return /(?:^|[/>:])braces(?:@[^/?#]*|\/[^?#]*)?(?:[?#].*)?$/.test(value);
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

export function assertNoDirectBraces(source: string): void {
  const literal = /"((?:\\[\s\S]|[^"\\])*)"|'((?:\\[\s\S]|[^'\\])*)'|`((?:\\[\s\S]|[^`\\])*)`/g;
  const gap = String.raw`(?:\s|\/\*[\s\S]*?\*\/|\/\/[^\r\n]*(?:\r?\n|$))*`;
  const imports = new RegExp(
    "\\b(?:from|import|require)\\b" + gap + "(?:\\(" + gap + ")?" + literal.source,
    "g",
  );
  for (const match of source.matchAll(imports)) {
    const specifier = cooked(match[1] ?? match[2] ?? match[3]);
    assert.ok(!protectedBracesModule(specifier), "unreviewed direct Braces literal: " + specifier);
  }
  for (const match of source.matchAll(literal)) {
    const value = cooked(match[1] ?? match[2] ?? match[3]);
    if (/^(?:https?:)?\/\//i.test(value))
      assert.ok(!protectedBracesModule(value), "new Braces CDN consumer");
  }
  const tags = /<script\b(?:[^"'<>]|"[^"]*"|'[^']*')*>/gi;
  const src = /(?:^|\s)src\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+))/gi;
  for (const tag of source.matchAll(tags))
    for (const attribute of tag[0].matchAll(src)) {
      let value = (attribute[1] ?? attribute[2] ?? attribute[3])
        .trim()
        .replaceAll("\t", "")
        .replaceAll("\r", "")
        .replaceAll("\n", "");
      assert.doesNotMatch(
        value,
        /&(?:#|[A-Za-z][A-Za-z0-9]*;)/,
        "character-reference script URL needs fresh review",
      );
      assert.ok(!protectedBracesModule(value), "unreviewed Braces script literal");
    }
}
