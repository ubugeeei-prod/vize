import { createHash } from "node:crypto";

interface BaselineComponentFinding {
  readonly file?: string;
  readonly ruleId: string;
  readonly message: string;
}

interface PatinaComponentFinding {
  readonly file: string;
  readonly ruleId: string;
  readonly severity: string;
  readonly line: number | bigint;
  readonly column: number | bigint;
  readonly endLine: number | bigint;
  readonly endColumn: number | bigint;
  readonly message: string;
}

export interface ReviewedPatinaComponentFinding extends PatinaComponentFinding {
  readonly project: string;
  readonly sourceSha256: string;
  readonly reason: string;
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
    if (!executableOffset(source, match.index ?? 0)) continue;
    let open = skipTrivia(source, (match.index ?? 0) + match[0].length);
    if (source[open] !== "(") continue;
    open = skipTrivia(source, open + 1);
    if (source[open] !== "{") continue;
    const close = matchingBrace(source, open);
    if (close !== null) objects.push([open, close]);
  }
  return objects;
}

function executableOffset(source: string, offset: number): boolean {
  for (let index = 0; index < offset;) {
    const character = source[index];
    if (character === '"' || character === "'" || character === "`") {
      const result = quoted(source, index, source.length);
      if (result === null) return false;
      if (offset < result[1]) return false;
      index = result[1];
    } else if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2),
        end = newline < 0 ? source.length : newline + 1;
      if (offset < end) return false;
      index = end;
    } else if (source.startsWith("/*", index)) {
      const close = source.indexOf("*/", index + 2),
        end = close < 0 ? source.length : close + 2;
      if (offset < end) return false;
      index = end;
    } else if (source.startsWith("<!--", index)) {
      const close = source.indexOf("-->", index + 4),
        end = close < 0 ? source.length : close + 3;
      if (offset < end) return false;
      index = end;
    } else index += 1;
  }
  return true;
}

function nextProperty(source: string, start: number, close: number): number {
  let depth = 0;
  for (let index = start; index < close; index += 1) {
    const character = source[index];
    if (character === '"' || character === "'" || character === "`") {
      const result = quoted(source, index, close);
      if (result === null) return close;
      index = result[1] - 1;
    } else if (source.startsWith("//", index) || source.startsWith("/*", index)) {
      index = skipTrivia(source, index, close) - 1;
    } else if (character === "{" || character === "[" || character === "(") depth += 1;
    else if (character === "}" || character === "]" || character === ")") depth -= 1;
    else if (character === "," && depth === 0) return index + 1;
  }
  return close;
}

function componentMapRegisters(source: string, open: number, close: number, tag: string): boolean {
  for (let cursor = open + 1; cursor < close;) {
    cursor = skipTrivia(source, cursor, close);
    if (source[cursor] === ",") {
      cursor += 1;
      continue;
    }
    const literal = quoted(source, cursor, close);
    let key: string;
    if (literal !== null) {
      if (literal[2]) return false;
      [key, cursor] = literal;
    } else {
      const identifier = /^[A-Za-z_$][\w$]*/u.exec(source.slice(cursor, close));
      if (identifier === null) return false;
      key = identifier[0];
      cursor += key.length;
    }
    if (comparableName(key) === comparableName(tag)) return true;
    cursor = nextProperty(source, cursor, close);
  }
  return false;
}

const comparableName = (name: string) => name.replaceAll("-", "").toLowerCase();

const reviewedPatinaComponentFindings: readonly ReviewedPatinaComponentFinding[] = [
  {
    project: "alexandrie",
    file: "frontend/app/pages/(public)/doc/[id]/index.vue",
    sourceSha256: "92fa7c5337036fdb84e5e37e713c3dbb2d713818709c5f6a86395451a65211d4",
    ruleId: "vue/require-component-registration",
    severity: "warning",
    line: 29,
    column: 6,
    endLine: 29,
    endColumn: 11,
    message:
      "[vize:vue/require-component-registration] Component is used but not explicitly imported",
    reason:
      "Alexandrie's pinned Nuxt app auto-registers app/components/Error.vue for this exact authored <Error> packet; eslint-plugin-vue instead treats Error as an in-scope JavaScript global.",
  },
  {
    project: "alexandrie",
    file: "frontend/app/pages/dashboard/docs/[id].vue",
    sourceSha256: "56ceeed14f2ea09f9410b9510dba0477e3a2c3efcdd00d8bfec660378f4d9d59",
    ruleId: "vue/require-component-registration",
    severity: "warning",
    line: 16,
    column: 6,
    endLine: 16,
    endColumn: 11,
    message:
      "[vize:vue/require-component-registration] Component is used but not explicitly imported",
    reason:
      "Alexandrie's pinned Nuxt app auto-registers app/components/Error.vue for this exact authored <Error> packet; eslint-plugin-vue instead treats Error as an in-scope JavaScript global.",
  },
];

function exactInteger(value: number | bigint): bigint | null {
  if (typeof value === "bigint") return value;
  return Number.isSafeInteger(value) ? BigInt(value) : null;
}

const sameInteger = (actual: number | bigint, expected: number | bigint) => {
  const left = exactInteger(actual),
    right = exactInteger(expected);
  return left !== null && right !== null && left === right;
};

export function componentRegistrationPatinaIntentionalReason(
  finding: PatinaComponentFinding,
  source: string,
  project: string,
  reviewed: readonly ReviewedPatinaComponentFinding[] = reviewedPatinaComponentFindings,
): string | null {
  if (finding.ruleId !== "vue/require-component-registration") return null;
  const sourceSha256 = createHash("sha256").update(source).digest("hex"),
    match = reviewed.find(
      (entry) =>
        entry.project === project &&
        entry.file === finding.file &&
        entry.sourceSha256 === sourceSha256 &&
        entry.ruleId === finding.ruleId &&
        entry.severity === finding.severity &&
        sameInteger(entry.line, finding.line) &&
        sameInteger(entry.column, finding.column) &&
        sameInteger(entry.endLine, finding.endLine) &&
        sameInteger(entry.endColumn, finding.endColumn) &&
        entry.message === finding.message,
    );
  return match?.reason ?? null;
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
    if (componentMapRegisters(source, components, componentsClose, tag)) return true;
  }
  return false;
}

function optionsApiRegistersNumericKebab(source: string, tag: string): boolean {
  if (!/-\d/u.test(tag)) return false;
  for (const match of source.matchAll(/\bexport\s+default\b/gu)) {
    if (!executableOffset(source, match.index ?? 0)) continue;
    const open = skipTrivia(source, (match.index ?? 0) + match[0].length);
    if (source[open] !== "{") continue;
    const close = matchingBrace(source, open);
    if (close === null) continue;
    const components = propertyValue(source, open, close, "components");
    if (components === null || source[components] !== "{") continue;
    const componentsClose = matchingBrace(source, components, close);
    if (componentsClose === null) continue;
    if (componentMapRegisters(source, components, componentsClose, tag)) return true;
  }
  return false;
}

/**
 * Vue's template compiler names an SFC after its file (`capitalize(camelize(base))`)
 * and resolves a tag with that name to the component itself when nothing else
 * registers it.
 */
function fileNamesSelf(file: string | undefined, tag: string): boolean {
  const base = /([^/\\]+)\.vue$/u.exec(file ?? "")?.[1];
  if (base === undefined) return false;
  const selfName = (name: string) => {
    const camel = name.replace(/-(\w)/gu, (_, character: string) => character.toUpperCase());
    return camel.charAt(0).toUpperCase() + camel.slice(1);
  };
  return selfName(tag) === selfName(base);
}

export function componentRegistrationBaselineIntentionalReason(
  finding: BaselineComponentFinding,
  source: string,
): string | null {
  if (finding.ruleId !== "vue/no-undef-components") return null;
  const tag = componentMessage.exec(finding.message)?.[1];
  if (tag === undefined) return null;
  if (fileNamesSelf(finding.file, tag))
    return "The tag names the component's own file, which Vue's template compiler resolves as a self-reference; eslint-plugin-vue does not read the file name.";
  if (defineOptionsNamesSelf(source, tag))
    return "The component renders itself recursively by its defineOptions name, which Vue resolves; eslint-plugin-vue does not read that name.";
  if (decoratorRegisters(source, tag))
    return "The vue-property-decorator components map registers this class component; eslint-plugin-vue does not associate that decorator registration with the template.";
  if (optionsApiRegistersNumericKebab(source, tag))
    return "Vue resolves this numeric kebab tag to its Options API registration key; eslint-plugin-vue does not associate that spelling with the registered component.";
  return null;
}
