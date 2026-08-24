export type RustToken = { value: string; start: number; end: number };

export function maskRange(output: string[], start: number, end: number): void {
  for (let index = start; index < end; index += 1) {
    if (output[index] !== "\n" && output[index] !== "\r") output[index] = " ";
  }
}

function charLiteralEnd(source: string, quote: number): number | undefined {
  let cursor = quote + 1;
  if (source[cursor] === "\\") {
    cursor += 1;
    if (source[cursor] === "u" && source[cursor + 1] === "{") {
      const brace = source.indexOf("}", cursor + 2);
      if (brace === -1) return undefined;
      cursor = brace + 1;
    } else if (source[cursor] === "x") cursor += 3;
    else cursor += 1;
  } else {
    const point = source.codePointAt(cursor);
    if (point === undefined || point === 0x0a || point === 0x0d || source[cursor] === "'") {
      return undefined;
    }
    cursor += point > 0xffff ? 2 : 1;
  }
  return source[cursor] === "'" ? cursor + 1 : undefined;
}

function quotedLiteralEnd(source: string, quote: number): number {
  let escaped = false;
  for (let cursor = quote + 1; cursor < source.length; cursor += 1) {
    const char = source[cursor];
    if (char === '"' && !escaped) return cursor + 1;
    escaped = char === "\\" && !escaped;
    if (char !== "\\") escaped = false;
  }
  return source.length;
}

function maskNonCode(source: string): string {
  const output = source.split("");
  let index = 0;
  while (index < source.length) {
    const pair = source.slice(index, index + 2);
    if (pair === "//") {
      const newline = source.indexOf("\n", index + 2);
      const end = newline === -1 ? source.length : newline;
      maskRange(output, index, end);
      index = end;
      continue;
    }
    if (pair === "/*") {
      let depth = 1;
      let cursor = index + 2;
      while (cursor < source.length && depth > 0) {
        const nested = source.slice(cursor, cursor + 2);
        if (nested === "/*") {
          depth += 1;
          cursor += 2;
        } else if (nested === "*/") {
          depth -= 1;
          cursor += 2;
        } else cursor += 1;
      }
      maskRange(output, index, cursor);
      index = cursor;
      continue;
    }
    const raw = source.slice(index).match(/^(?:br|r)(#*)"/u);
    if (raw) {
      const close = `"${raw[1]}`;
      const contentStart = index + raw[0].length;
      const closeAt = source.indexOf(close, contentStart);
      const end = closeAt === -1 ? source.length : closeAt + close.length;
      maskRange(output, index, end);
      index = end;
      continue;
    }
    const byteCharEnd = pair === "b'" ? charLiteralEnd(source, index + 1) : undefined;
    const charEnd = source[index] === "'" ? charLiteralEnd(source, index) : undefined;
    const literalEnd = byteCharEnd ?? charEnd;
    if (literalEnd !== undefined) {
      maskRange(output, index, literalEnd);
      index = literalEnd;
      continue;
    }
    const stringStart = source[index] === '"' ? 1 : pair === 'b"' ? 2 : 0;
    if (stringStart > 0) {
      const end = quotedLiteralEnd(source, index + stringStart - 1);
      maskRange(output, index, end);
      index = end;
      continue;
    }
    index += 1;
  }
  return output.join("");
}

function matchingBrace(source: string, open: number): number {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    else if (source[index] === "}" && --depth === 0) return index + 1;
  }
  return source.length;
}

export function maskRustSource(source: string): string {
  const code = maskNonCode(source);
  const output = code.split("");
  const cfg = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/gu;
  for (const match of code.matchAll(cfg)) {
    const start = match.index;
    let cursor = start + match[0].length;
    while (cursor < code.length && /\s/u.test(code[cursor])) cursor += 1;
    while (code.startsWith("#[", cursor)) {
      const end = code.indexOf("]", cursor + 2);
      cursor = end === -1 ? code.length : end + 1;
      while (cursor < code.length && /\s/u.test(code[cursor])) cursor += 1;
    }
    const semicolon = code.indexOf(";", cursor);
    const brace = code.indexOf("{", cursor);
    const end =
      brace !== -1 && (semicolon === -1 || brace < semicolon)
        ? matchingBrace(code, brace)
        : semicolon === -1
          ? code.length
          : semicolon + 1;
    maskRange(output, start, end);
  }
  return output.join("");
}

export function tokensOf(source: string): RustToken[] {
  const tokens: RustToken[] = [];
  const pattern = /r#[A-Za-z_]\w*|[A-Za-z_]\w*|::|[{};,*!]/gu;
  for (const match of source.matchAll(pattern)) {
    tokens.push({ value: match[0], start: match.index, end: match.index + match[0].length });
  }
  return tokens;
}

export function ident(token: RustToken | undefined): string | undefined {
  if (token === undefined || !/^(?:r#)?[A-Za-z_]\w*$/u.test(token.value)) return undefined;
  return token.value.replace(/^r#/u, "");
}
