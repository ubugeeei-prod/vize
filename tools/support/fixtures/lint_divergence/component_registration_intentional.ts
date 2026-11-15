interface BaselineComponentFinding {
  readonly ruleId: string;
  readonly message: string;
}

const componentMessage =
  /^The '<([A-Za-z_$][\w$-]*)>' component has been used, but not defined\.$/u;

function skipTrivia(source: string, start: number, end = source.length): number {
  let index = start;
  while (index < end) {
    if (/\s/u.test(source[index] ?? "")) {
      index += 1;
    } else if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      index = newline < 0 ? end : newline + 1;
    } else if (source.startsWith("/*", index)) {
      const close = source.indexOf("*/", index + 2);
      index = close < 0 ? end : close + 2;
    } else {
      break;
    }
  }
  return index;
}

function quoted(source: string, start: number, end: number): [string, number] | null {
  const quote = source[start];
  if (quote !== '"' && quote !== "'" && quote !== "`") return null;
  let value = "";
  for (let index = start + 1; index < end; index += 1) {
    const character = source[index];
    if (character === "\\") {
      const escaped = source[index + 1];
      if (escaped === undefined) return null;
      value += escaped;
      index += 1;
    } else if (character === quote) {
      return [value, index + 1];
    } else if (quote === "`" && character === "$" && source[index + 1] === "{") {
      return null;
    } else {
      value += character;
    }
  }
  return null;
}

function matchingBrace(source: string, open: number, end = source.length): number | null {
  let depth = 0;
  for (let index = open; index < end; index += 1) {
    const character = source[index];
    if (character === '"' || character === "'" || character === "`") {
      const result = quoted(source, index, end);
      if (result === null) return null;
      index = result[1] - 1;
    } else if (source.startsWith("//", index) || source.startsWith("/*", index)) {
      index = skipTrivia(source, index, end) - 1;
    } else if (character === "{") {
      depth += 1;
    } else if (character === "}" && --depth === 0) {
      return index;
    }
  }
  return null;
}

function propertyValue(source: string, open: number, close: number, wanted: string): number | null {
  for (let index = open + 1; index < close;) {
    index = skipTrivia(source, index, close);
    if (source[index] === ",") {
      index += 1;
      continue;
    }
    const literal = quoted(source, index, close);
    let key: string;
    if (literal !== null) {
      [key, index] = literal;
    } else {
      const identifier = /^[A-Za-z_$][\w$]*/u.exec(source.slice(index, close));
      if (identifier === null) return null;
      key = identifier[0];
      index += key.length;
    }
    index = skipTrivia(source, index, close);
    if (source[index] === ":") {
      index = skipTrivia(source, index + 1, close);
      if (key === wanted) return index;
    }
    let nested = 0;
    while (index < close) {
      const character = source[index];
      if (character === '"' || character === "'" || character === "`") {
        const result = quoted(source, index, close);
        if (result === null) return null;
        index = result[1];
        continue;
      }
      if (source.startsWith("//", index) || source.startsWith("/*", index)) {
        index = skipTrivia(source, index, close);
        continue;
      }
      if (character === "{" || character === "[" || character === "(") nested += 1;
      else if (character === "}" || character === "]" || character === ")") nested -= 1;
      else if (character === "," && nested === 0) break;
      index += 1;
    }
  }
  return null;
}

function objectArguments(source: string, callee: RegExp): Array<[number, number]> {
  const objects: Array<[number, number]> = [];
  for (const match of source.matchAll(callee)) {
    let open = skipTrivia(source, (match.index ?? 0) + match[0].length);
    if (source[open] !== "(") continue;
    open = skipTrivia(source, open + 1);
    if (source[open] !== "{") continue;
    const close = matchingBrace(source, open);
    if (close !== null) objects.push([open, close]);
  }
  return objects;
}

const comparableName = (name: string) => name.replaceAll("-", "").toLowerCase();

function defineOptionsNamesSelf(source: string, tag: string): boolean {
  for (const [open, close] of objectArguments(source, /\bdefineOptions\b/gu)) {
    const value = propertyValue(source, open, close, "name");
    if (value === null) continue;
    const literal = quoted(source, value, close);
    if (literal !== null && comparableName(literal[0]) === comparableName(tag)) return true;
  }
  return false;
}

function decoratorRegisters(source: string, tag: string): boolean {
  for (const [open, close] of objectArguments(source, /@Component\b/gu)) {
    const components = propertyValue(source, open, close, "components");
    if (components === null || source[components] !== "{") continue;
    const componentsClose = matchingBrace(source, components, close);
    if (componentsClose === null) continue;
    let cursor = components + 1;
    while (cursor < componentsClose) {
      cursor = skipTrivia(source, cursor, componentsClose);
      if (source[cursor] === ",") {
        cursor += 1;
        continue;
      }
      const literal = quoted(source, cursor, componentsClose);
      let key: string;
      if (literal !== null) [key, cursor] = literal;
      else {
        const identifier = /^[A-Za-z_$][\w$]*/u.exec(source.slice(cursor, componentsClose));
        if (identifier === null) break;
        key = identifier[0];
        cursor += key.length;
      }
      if (comparableName(key) === comparableName(tag)) return true;
      while (cursor < componentsClose && source[cursor] !== ",") cursor += 1;
    }
  }
  return false;
}

export function componentRegistrationBaselineIntentionalReason(
  finding: BaselineComponentFinding,
  source: string,
): string | null {
  if (finding.ruleId !== "vue/no-undef-components") return null;
  const tag = componentMessage.exec(finding.message)?.[1];
  if (tag === undefined) return null;
  if (defineOptionsNamesSelf(source, tag))
    return "The component renders itself recursively by its defineOptions name, which Vue resolves; eslint-plugin-vue does not read that name.";
  if (decoratorRegisters(source, tag))
    return "The vue-property-decorator components map registers this class component; eslint-plugin-vue does not associate that decorator registration with the template.";
  return null;
}
