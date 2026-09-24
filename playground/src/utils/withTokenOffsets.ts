type TextToken = { text: string } | { content: string };

/** Give display-only syntax tokens a key tied to their span within a line. */
export function withTokenOffsets<T extends TextToken>(
  tokens: readonly T[],
): Array<T & { offset: number }> {
  let offset = 0;
  return tokens.map((token) => {
    const positioned = { ...token, offset };
    offset += "text" in token ? token.text.length : token.content.length;
    return positioned;
  });
}
