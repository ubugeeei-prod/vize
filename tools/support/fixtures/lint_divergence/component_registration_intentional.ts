import fs from "node:fs";
import path from "node:path";

interface BaselineComponentFinding {
  readonly ruleId: string;
  readonly message: string;
}

interface PatinaComponentFinding {
  readonly file: string;
  readonly ruleId: string;
  readonly line: number | bigint;
  readonly column: number | bigint;
  readonly endLine: number | bigint;
  readonly endColumn: number | bigint;
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

function templateExpressionEnd(source: string, open: number, end: number): number | null {
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

function quoted(source: string, start: number, end: number): [string, number, boolean] | null {
  const quote = source[start];
  if (quote !== '"' && quote !== "'" && quote !== "`") return null;
  let value = "",
    interpolated = false;
  for (let index = start + 1; index < end; index += 1) {
    const character = source[index];
    if (character === "\\") {
      const escaped = source[index + 1];
      if (escaped === undefined) return null;
      value += escaped;
      index += 1;
    } else if (character === quote) {
      return [value, index + 1, interpolated];
    } else if (quote === "`" && character === "$" && source[index + 1] === "{") {
      const close = templateExpressionEnd(source, index + 1, end);
      if (close === null) return null;
      value += source.slice(index, close + 1);
      interpolated = true;
      index = close;
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
      if (literal[2]) return null;
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

function authoredFindingText(finding: PatinaComponentFinding, source: string): string | null {
  const line = Number(finding.line),
    endLine = Number(finding.endLine),
    column = Number(finding.column),
    endColumn = Number(finding.endColumn);
  if (
    !Number.isSafeInteger(line) ||
    !Number.isSafeInteger(endLine) ||
    !Number.isSafeInteger(column) ||
    !Number.isSafeInteger(endColumn) ||
    line !== endLine ||
    line < 1 ||
    column < 1 ||
    endColumn < column
  )
    return null;
  const authoredLine = source.split("\n")[line - 1];
  return authoredLine?.slice(column - 1, endColumn - 1) ?? null;
}

function regularFile(filename: string): boolean {
  try {
    return fs.statSync(filename).isFile();
  } catch {
    return false;
  }
}

const nuxtConfigNames = [
  "nuxt.config.ts",
  "nuxt.config.js",
  "nuxt.config.mjs",
  "nuxt.config.cjs",
  "nuxt.config.mts",
  "nuxt.config.cts",
];

function nearestNuxtRoot(cwd: string, file: string): string | null {
  const workspace = path.resolve(cwd);
  let current = path.dirname(path.resolve(workspace, file));
  for (;;) {
    if (nuxtConfigNames.some((name) => regularFile(path.join(current, name)))) return current;
    if (current === workspace) return null;
    const parent = path.dirname(current);
    const relative = path.relative(workspace, parent);
    if (
      parent === current ||
      path.isAbsolute(relative) ||
      relative === ".." ||
      relative.startsWith(`..${path.sep}`)
    )
      return null;
    current = parent;
  }
}

export function componentRegistrationPatinaIntentionalReason(
  finding: PatinaComponentFinding,
  source: string,
  cwd: string,
): string | null {
  if (
    finding.ruleId !== "vue/require-component-registration" ||
    authoredFindingText(finding, source) !== "Error"
  )
    return null;
  const nuxtRoot = nearestNuxtRoot(cwd, finding.file);
  if (nuxtRoot === null) return null;
  const componentExists = ["app/components/Error.vue", "components/Error.vue"].some((file) =>
    regularFile(path.join(nuxtRoot, file)),
  );
  if (!componentExists) return null;
  return "The nearest Nuxt app auto-registers its authored Error.vue component for the <Error> tag; eslint-plugin-vue instead treats Error as an in-scope JavaScript global.";
}

function defineOptionsNamesSelf(source: string, tag: string): boolean {
  for (const [open, close] of objectArguments(source, /\bdefineOptions\b/gu)) {
    const value = propertyValue(source, open, close, "name");
    if (value === null) continue;
    const literal = quoted(source, value, close);
    if (literal !== null && !literal[2] && comparableName(literal[0]) === comparableName(tag))
      return true;
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
      if (literal !== null) {
        if (literal[2]) break;
        [key, cursor] = literal;
      } else {
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

function optionsApiRegistersNumericKebab(source: string, tag: string): boolean {
  if (!/-\d/u.test(tag)) return false;
  for (const match of source.matchAll(/\bexport\s+default\b/gu)) {
    const open = skipTrivia(source, (match.index ?? 0) + match[0].length);
    if (source[open] !== "{") continue;
    const close = matchingBrace(source, open);
    if (close === null) continue;
    const components = propertyValue(source, open, close, "components");
    if (components === null || source[components] !== "{") continue;
    const componentsClose = matchingBrace(source, components, close);
    if (componentsClose === null) continue;
    for (let cursor = components + 1; cursor < componentsClose;) {
      cursor = skipTrivia(source, cursor, componentsClose);
      if (source[cursor] === ",") {
        cursor += 1;
        continue;
      }
      const literal = quoted(source, cursor, componentsClose);
      let key: string;
      if (literal !== null) {
        if (literal[2]) break;
        [key, cursor] = literal;
      } else {
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
  if (optionsApiRegistersNumericKebab(source, tag))
    return "Vue resolves this numeric kebab tag to its Options API registration key; eslint-plugin-vue does not associate that spelling with the registered component.";
  return null;
}
