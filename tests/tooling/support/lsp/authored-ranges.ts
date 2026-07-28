import assert from "node:assert/strict";
import type { LspPosition, LspRange } from "./protocol.ts";

/**
 * Helpers for pinning LSP responses to *authored* `.vue` coordinates.
 *
 * `vize lsp` answers most requests by querying a generated virtual TypeScript
 * document and mapping the result back to the SFC the user actually typed. A
 * mapping regression shows up as a whole-line or whole-column shift, which
 * survives every `length > 0` / `.some(...)` style assertion. These helpers let
 * suites state the authored span exactly and prove the returned range really
 * addresses the text it claims to (#2971 audit item 8).
 */

export type SemanticToken = {
  line: number;
  character: number;
  length: number;
  tokenType: string;
  tokenModifiers: string[];
};

export type SemanticTokensLegend = {
  tokenTypes: string[];
  tokenModifiers: string[];
};

export type LspLocation = {
  uri: string;
  range: LspRange;
};

/** Builds a single-line authored range, the shape every identifier hit takes. */
export function authoredRange(
  line: number,
  startCharacter: number,
  endCharacter: number,
): LspRange {
  return {
    start: authoredPosition(line, startCharacter),
    end: authoredPosition(line, endCharacter),
  };
}

export function authoredPosition(line: number, character: number): LspPosition {
  return { line, character };
}

/** Key-order independent range equality; servers may serialize either order. */
export function sameRange(left: LspRange, right: LspRange): boolean {
  return (
    left.start.line === right.start.line &&
    left.start.character === right.start.character &&
    left.end.line === right.end.line &&
    left.end.character === right.end.character
  );
}

/**
 * Asserts that every range addresses text that exists in the authored source.
 *
 * A range that survives this check cannot have leaked raw virtual-document
 * coordinates past the end of the line it names.
 */
export function assertAuthoredRanges(source: string, ranges: LspRange[]): void {
  const lines = source.split("\n");
  for (const value of ranges) {
    assert.equal(value.start.line, value.end.line, JSON.stringify(value));
    const line = lines[value.start.line];
    assert.ok(line != null, `line ${value.start.line} is outside the authored file`);
    assert.ok(
      value.end.character <= line.length,
      `${JSON.stringify(value)} overruns authored line ${JSON.stringify(line)}`,
    );
    assert.ok(value.start.character < value.end.character, JSON.stringify(value));
  }
}

/** Asserts the authored text each decoded token covers, in order. */
export function assertTokenText(source: string, tokens: SemanticToken[], expected: string[]): void {
  const lines = source.split("\n");
  assert.deepEqual(
    tokens.map((token) =>
      lines[token.line]?.slice(token.character, token.character + token.length),
    ),
    expected,
  );
}

export function semanticToken(
  line: number,
  character: number,
  length: number,
  tokenType: string,
  tokenModifiers: string[] = [],
): SemanticToken {
  return { line, character, length, tokenType, tokenModifiers };
}

/** Reads the legend the server advertised so tokens decode to names, not indices. */
export function semanticTokensLegend(initializeResult: unknown): SemanticTokensLegend {
  const provider = (initializeResult as { capabilities?: { semanticTokensProvider?: unknown } })
    .capabilities?.semanticTokensProvider as { legend?: SemanticTokensLegend } | undefined;
  assert.ok(provider?.legend, "server must advertise a semantic tokens legend");
  return provider.legend;
}

/** Expands the LSP 5-tuple delta encoding into absolute authored positions. */
export function decodeSemanticTokens(
  response: { data?: number[] } | null,
  legend: SemanticTokensLegend,
): SemanticToken[] {
  assert.ok(Array.isArray(response?.data), JSON.stringify(response));
  const data = response.data;
  assert.equal(data.length % 5, 0, JSON.stringify(data));

  const tokens: SemanticToken[] = [];
  let line = 0;
  let character = 0;
  for (let index = 0; index < data.length; index += 5) {
    const [deltaLine, deltaStart, length, typeIndex, modifierBits] = data.slice(index, index + 5);
    line += deltaLine;
    character = deltaLine === 0 ? character + deltaStart : deltaStart;
    const tokenType = legend.tokenTypes[typeIndex];
    assert.ok(tokenType, `token type ${typeIndex} is outside the advertised legend`);
    tokens.push({
      line,
      character,
      length,
      tokenType,
      tokenModifiers: legend.tokenModifiers.filter((_, bit) => (modifierBits & (1 << bit)) !== 0),
    });
  }
  return tokens;
}
