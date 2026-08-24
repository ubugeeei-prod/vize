/** Extract object-literal generic bodies from calls such as `defineEmits<{ ... }>()`. */
export function typeLiteralCallBodies(source: string, callee: string): string[] {
  const bodies: string[] = [];
  let cursor = 0;

  while (cursor < source.length) {
    const found = source.indexOf(callee, cursor);
    if (found === -1) break;

    cursor = found + callee.length;
    if (isIdentifierPart(source[found - 1]) || isIdentifierPart(source[cursor])) continue;

    let index = skipTrivia(source, cursor);
    if (source[index] !== "<") continue;
    index = skipTrivia(source, index + 1);
    if (source[index] !== "{") continue;

    const end = findMatchingObject(source, index);
    if (end === undefined) continue;

    const afterObject = skipTrivia(source, end + 1);
    const afterType = skipTrivia(source, afterObject + 1);
    if (source[afterObject] === ">" && source[afterType] === "(") {
      bodies.push(source.slice(index + 1, end));
    }
    cursor = Math.max(cursor, afterType + 1);
  }

  return bodies;
}

/** Find generic calls whose type argument is not an inline object literal. */
export function nonLiteralTypeArgumentCalls(source: string, callee: string): string[] {
  const typeArguments: string[] = [];
  let cursor = 0;

  while (cursor < source.length) {
    const found = source.indexOf(callee, cursor);
    if (found === -1) break;

    cursor = found + callee.length;
    if (isIdentifierPart(source[found - 1]) || isIdentifierPart(source[cursor])) continue;

    const genericStart = skipTrivia(source, cursor);
    if (source[genericStart] !== "<") continue;

    const typeStart = skipTrivia(source, genericStart + 1);
    if (source[typeStart] === "{") {
      cursor = typeStart + 1;
      continue;
    }

    const typeEnd = findMatchingTypeArgument(source, genericStart);
    if (typeEnd === undefined) continue;

    const afterType = skipTrivia(source, typeEnd + 1);
    if (source[afterType] === "(") {
      typeArguments.push(source.slice(typeStart, typeEnd).trim());
    }
    cursor = Math.max(cursor, afterType + 1);
  }

  return typeArguments;
}

/** Split only top-level members; nested tuple/object payload fields stay inside one member. */
export function splitTopLevelTypeMembers(body: string): string[] {
  const members: string[] = [];
  let start = 0;
  let depth = 0;
  let quote: '"' | "'" | "`" | undefined;
  let inBlockComment = false;
  let inLineComment = false;

  for (let index = 0; index < body.length; index += 1) {
    const char = body[index];
    const next = body[index + 1];

    if (inLineComment) {
      if (char === "\n") inLineComment = false;
      continue;
    }
    if (inBlockComment) {
      if (char === "*" && next === "/") {
        inBlockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote !== undefined) {
      if (char === "\\") index += 1;
      else if (char === quote) quote = undefined;
      continue;
    }
    if (char === "/" && next === "/") {
      inLineComment = true;
      index += 1;
      continue;
    }
    if (char === "/" && next === "*") {
      inBlockComment = true;
      index += 1;
      continue;
    }
    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }

    if (char === "{" || char === "[" || char === "(" || char === "<") {
      depth += 1;
      continue;
    }
    if (char === "}" || char === "]" || char === ")" || char === ">") {
      depth = Math.max(0, depth - 1);
      continue;
    }
    if (depth === 0 && (char === ";" || char === ",")) {
      members.push(body.slice(start, index));
      start = index + 1;
    }
  }

  members.push(body.slice(start));
  return members.filter((member) => member.trim().length > 0);
}

function findMatchingObject(source: string, openIndex: number): number | undefined {
  let depth = 0;
  let quote: '"' | "'" | "`" | undefined;
  let inBlockComment = false;
  let inLineComment = false;

  for (let index = openIndex; index < source.length; index += 1) {
    const char = source[index];
    const next = source[index + 1];

    if (inLineComment) {
      if (char === "\n") inLineComment = false;
      continue;
    }
    if (inBlockComment) {
      if (char === "*" && next === "/") {
        inBlockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote !== undefined) {
      if (char === "\\") index += 1;
      else if (char === quote) quote = undefined;
      continue;
    }
    if (char === "/" && next === "/") {
      inLineComment = true;
      index += 1;
      continue;
    }
    if (char === "/" && next === "*") {
      inBlockComment = true;
      index += 1;
      continue;
    }
    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }
    if (char === "{") depth += 1;
    if (char === "}" && --depth === 0) return index;
  }

  return undefined;
}

function findMatchingTypeArgument(source: string, openIndex: number): number | undefined {
  let depth = 0;
  let quote: '"' | "'" | "`" | undefined;
  let inBlockComment = false;
  let inLineComment = false;

  for (let index = openIndex; index < source.length; index += 1) {
    const char = source[index];
    const next = source[index + 1];

    if (inLineComment) {
      if (char === "\n") inLineComment = false;
      continue;
    }
    if (inBlockComment) {
      if (char === "*" && next === "/") {
        inBlockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote !== undefined) {
      if (char === "\\") index += 1;
      else if (char === quote) quote = undefined;
      continue;
    }
    if (char === "/" && next === "/") {
      inLineComment = true;
      index += 1;
      continue;
    }
    if (char === "/" && next === "*") {
      inBlockComment = true;
      index += 1;
      continue;
    }
    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }
    if (char === "<") {
      depth += 1;
      continue;
    }
    if (char === ">" && source[index - 1] !== "=") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }

  return undefined;
}

function skipTrivia(source: string, index: number): number {
  let cursor = index;
  while (cursor < source.length) {
    if (/\s/u.test(source[cursor] ?? "")) {
      cursor += 1;
    } else if (source[cursor] === "/" && source[cursor + 1] === "/") {
      cursor = source.indexOf("\n", cursor + 2);
      if (cursor === -1) return source.length;
    } else if (source[cursor] === "/" && source[cursor + 1] === "*") {
      const end = source.indexOf("*/", cursor + 2);
      if (end === -1) return source.length;
      cursor = end + 2;
    } else {
      return cursor;
    }
  }
  return cursor;
}

function isIdentifierPart(char: string | undefined): boolean {
  return char !== undefined && /[$\w]/u.test(char);
}
