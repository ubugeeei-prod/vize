// Props that only an imported contract declares.
//
// The eslint-plugin-vue baseline runs without type-aware parser services, so a
// `defineProps<T>()` member that T receives from another module, or a runtime
// props object imported into `defineProps(options)`, is unknown to it. Patina
// resolves that module and checks the prop. A finding is accepted here only
// when it sits on such a prop: the SFC's scripts never declare the name, and
// the props reach an import binding through local aliases and interface
// heritage. A prop the SFC declares itself stays counted.
//
// Defaults rules (`require-default-prop`, `no-boolean-default`) report inside
// the props call. Collision rules (`no-dupe-keys`, `no-template-shadow`)
// report the binding or template variable that reuses the prop's name.

export const importedPropContractReason =
  "Patina resolves optional props declared by a type contract imported from another module; the eslint-plugin-vue baseline runs without type-aware parser services, so those props are unknown to it.";

export const importedPropCollisionReason =
  "Patina resolves props declared by a contract imported from another module, so a same-named binding or template variable collides with one; the eslint-plugin-vue baseline runs without type-aware parser services, so those props are unknown to it.";

interface Range {
  start: number;
  end: number;
}

interface PropsCall {
  typeArgument: Range;
  defaults: Range | null;
}

const identifier = /[A-Za-z_$][\w$]*/gu;
const maximumVisitedTypes = 64;

/**
 * A finding on the `defineProps<T>` type argument that names the prop:
 * `require-default-prop`, `no-required-prop-with-default` and
 * `prop-name-casing` report there.
 */
export function propFindingFromImportedContract(
  message: string,
  start: number,
  end: number,
  source: string,
): boolean {
  const name = /Prop (?:'([^']+)'|"([^"]+)")/u.exec(message)?.slice(1).find(Boolean);
  if (name === undefined) return false;
  const scripts = scriptBlocks(source);
  return propsCalls(source).some(
    (call) =>
      call.typeArgument.start <= start &&
      end <= call.typeArgument.end &&
      importedOnly(source, scripts, call, name),
  );
}

/** `no-boolean-default` reports on the default value of a defaults entry. */
export function booleanDefaultFromImportedContract(
  start: number,
  end: number,
  source: string,
): boolean {
  const scripts = scriptBlocks(source);
  return propsCalls(source).some((call) => {
    const defaults = call.defaults;
    if (defaults === null || start < defaults.start || defaults.end < end) return false;
    const name = defaultEntryKey(source.slice(defaults.start, start));
    return name !== null && importedOnly(source, scripts, call, name);
  });
}

/**
 * `no-dupe-keys` reports the binding or `defineModel` name that collides
 * with a prop. Accepted only when the prop comes from an import and the
 * finding's statement holds every same-file declaration of the name.
 */
export function duplicateKeyFromImportedContract(
  message: string,
  start: number,
  end: number,
  source: string,
): boolean {
  const name = /^\[vize:[^\]]+\] Duplicated key '([^']+)'/u.exec(message)?.[1];
  if (name === undefined) return false;
  const scripts = scriptBlocks(source);
  const statement = statementAt(source, scripts, start);
  if (statement === null || end > statement.end || !importedPropOnly(source, scripts, name))
    return false;
  const sites = declarationSites(source, scripts, name);
  return (
    sites.some((site) => start <= site && site < end) &&
    sites.every((site) => statement.start <= site && site < statement.end)
  );
}

/**
 * `no-template-shadow` reports the template variable. Accepted only when the
 * shadowed prop comes from an import and the scripts declare nothing else
 * under that name.
 */
export function templateShadowFromImportedContract(
  message: string,
  start: number,
  source: string,
): boolean {
  const name = /^\[vize:[^\]]+\] Variable '([^']+)' shadows/u.exec(message)?.[1];
  if (name === undefined || !source.startsWith(name, start)) return false;
  const scripts = scriptBlocks(source);
  return (
    !scripts.some((script) => script.start <= start && start < script.end) &&
    importedPropOnly(source, scripts, name) &&
    declarationSites(source, scripts, name).length === 0
  );
}

/** No same-file type member or runtime key declares `name`, and the props reach an import. */
function importedPropOnly(source: string, scripts: Range[], name: string): boolean {
  // Slot and emit interfaces often repeat a prop name (`open`), so only the
  // types the props reach count here.
  const reached = new Set(
    propsCalls(source).flatMap((call) => [
      ...typeClosure(source, source.slice(call.typeArgument.start, call.typeArgument.end)),
    ]),
  );
  if (
    declaresPropName(
      source,
      typeRegions(source, scripts, (typeName) => reached.has(typeName)),
      name,
    ) ||
    runtimeKeyRegions(source).some(mentions(source, name))
  )
    return false;
  const imports = importBindings(source, scripts);
  return (
    propsCalls(source).some((call) =>
      reachesImport(source, scripts, source.slice(call.typeArgument.start, call.typeArgument.end)),
    ) ||
    [...source.matchAll(/\bdefineProps\s*\(\s*([A-Za-z_$][\w$]*)\s*\)/gu)].some((match) =>
      imports.has(match[1] ?? ""),
    )
  );
}

/**
 * Object and array literals that can declare runtime props: a `defineProps`
 * argument or a `props` option. A `(props: { ... })` parameter type is neither.
 */
function runtimeKeyRegions(source: string): Range[] {
  const regions: Range[] = [];
  for (const match of source.matchAll(/(?:\bdefineProps\s*\(|(?:^|[{,])\s*props\s*:)\s*[{[]/gmu)) {
    const open = (match.index ?? 0) + match[0].length - 1;
    regions.push({ start: open, end: matchingBracket(source, open) ?? source.length });
  }
  return regions;
}

function mentions(source: string, name: string): (region: Range) => boolean {
  const word = new RegExp(`(?:^|[^\\w$])${escapeName(name)}(?![\\w$])`, "u");
  return (region) => word.test(source.slice(region.start, region.end));
}

/**
 * Every script offset that can declare `name`: `const`/`let`/`var`,
 * `function` and `class` names, destructure patterns, import clauses and
 * `defineModel` names (`modelValue` for an unnamed model). Nested scopes
 * count too, so a doubtful name is never accepted. Only an object pattern
 * that reads the props object back (`const { tab, ...rest } = props`) is
 * skipped: its names are the props themselves.
 */
function declarationSites(source: string, scripts: Range[], name: string): number[] {
  const escaped = escapeName(name);
  const sites: number[] = [];
  const props =
    /\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*=\s*(?:withDefaults\s*\(\s*)?defineProps\b/u.exec(
      source,
    )?.[1];
  const readsProps =
    props === undefined ? null : new RegExp(`^\\}\\s*=\\s*${escapeName(props)}(?![\\w$.?[])`, "u");
  const direct = new RegExp(
    `\\b(?:const|let|var|function\\s*\\*?|class)\\s+(${escaped})(?![\\w$])`,
    "gu",
  );
  const model = new RegExp(`\\bdefineModel\\s*(?:<[^()]*>)?\\s*\\(\\s*(['"])${escaped}\\1`, "gu");
  // A renamed destructure key (`name: local`) or import (`name as local`) binds another name.
  const word = new RegExp(`(?:^|[^\\w$.])(${escaped})(?![\\w$])(?!\\s*:)(?!\\s+as\\b)`, "gu");
  for (const script of scripts) {
    const text = source.slice(script.start, script.end);
    for (const match of text.matchAll(direct))
      sites.push(script.start + (match.index ?? 0) + match[0].length - name.length);
    for (const match of text.matchAll(model))
      sites.push(script.start + (match.index ?? 0) + match[0].length - name.length - 1);
    if (name === "modelValue")
      for (const match of text.matchAll(/\bdefineModel\s*(?:<[^()]*>)?\s*\(\s*(?!['"\s])/gu))
        sites.push(script.start + (match.index ?? 0));
    const patterns = [
      ...text.matchAll(/\b(?:const|let|var)\s*[{[]/gu),
      ...text.matchAll(/\bimport\s+(?:type\s+)?[^'";]*?\s+from\s*['"]/gu),
    ];
    for (const pattern of patterns) {
      const begin = script.start + (pattern.index ?? 0);
      const open = begin + pattern[0].length - 1;
      const close =
        pattern[0].endsWith("[") || pattern[0].endsWith("{")
          ? (matchingBracket(source, open) ?? script.end)
          : open;
      if (pattern[0].endsWith("{") && readsProps?.test(source.slice(close, close + 80)) === true)
        continue;
      for (const match of source.slice(begin, close).matchAll(word))
        sites.push(begin + (match.index ?? 0) + match[0].length - name.length);
    }
  }
  return [...new Set(sites)].sort((left, right) => left - right);
}

/** The top-level declaration statement whose text contains `offset`. */
function statementAt(source: string, scripts: Range[], offset: number): Range | null {
  const script = scripts.find((block) => block.start <= offset && offset < block.end);
  if (script === undefined) return null;
  const text = source.slice(script.start, offset);
  let start: number | null = null;
  for (const match of text.matchAll(/^[ \t]*(?:export\s+)?(?:const|let|var)\b/gmu))
    start = script.start + (match.index ?? 0);
  if (start === null) return null;
  const end = aliasEnd(source, start, script.end);
  return end < offset ? null : { start, end };
}

export function escapeName(name: string): string {
  return name.replace(/[.*+?^${}()|[\]\\]/gu, (character) => `\\${character}`);
}

function importedOnly(source: string, scripts: Range[], call: PropsCall, name: string): boolean {
  const argument = source.slice(call.typeArgument.start, call.typeArgument.end);
  return (
    !declaresPropName(source, typeRegions(source, scripts), name) &&
    // A PascalCase "prop" named after a type the argument references is not a prop.
    !(/^[A-Z]/u.test(name) && typeClosure(source, argument).has(name)) &&
    reachesImport(source, scripts, argument)
  );
}

export function scriptBlocks(source: string): Range[] {
  const blocks: Range[] = [];
  for (const match of source.matchAll(/<script\b[^>]*>/giu)) {
    const start = (match.index ?? 0) + match[0].length;
    const close = source.indexOf("</script>", start);
    blocks.push({ start, end: close < 0 ? source.length : close });
  }
  return blocks;
}

/** Every `defineProps<T>()` with its `withDefaults` object or destructure pattern. */
function propsCalls(source: string): PropsCall[] {
  const calls: PropsCall[] = [];
  for (const match of source.matchAll(/\bdefineProps\s*</gu)) {
    const open = (match.index ?? 0) + match[0].length;
    const close = matchingAngle(source, open);
    if (close === null) continue;
    const callOpen = skipSpace(source, close + 1);
    if (source[callOpen] !== "(") continue;
    const callClose = matchingBracket(source, callOpen);
    if (callClose === null) continue;
    calls.push({
      typeArgument: { start: open, end: close },
      defaults:
        withDefaultsObject(source, match.index ?? 0, callClose) ??
        destructurePattern(source, match.index ?? 0),
    });
  }
  return calls;
}

function withDefaultsObject(source: string, propsStart: number, callClose: number): Range | null {
  if (!/\bwithDefaults\s*\(\s*$/u.test(source.slice(Math.max(0, propsStart - 200), propsStart)))
    return null;
  const comma = skipSpace(source, callClose + 1);
  if (source[comma] !== ",") return null;
  const open = skipSpace(source, comma + 1);
  if (source[open] !== "{") return null;
  const close = matchingBracket(source, open);
  return close === null ? null : { start: open + 1, end: close };
}

function destructurePattern(source: string, propsStart: number): Range | null {
  const prefix = source.slice(0, propsStart);
  const assignment = /\}\s*=\s*$/u.exec(prefix);
  if (assignment === null) return null;
  const close = assignment.index;
  let depth = 0;
  for (let index = close; index >= 0; index--) {
    const character = prefix[index];
    if (character === "}") depth++;
    else if (character === "{" && --depth === 0) {
      return /\b(?:const|let|var)\s*$/u.test(prefix.slice(Math.max(0, index - 20), index))
        ? { start: index + 1, end: close }
        : null;
    }
  }
  return null;
}

/** The authored key of the defaults entry whose value starts the finding. */
function defaultEntryKey(before: string): string | null {
  const key = String.raw`([A-Za-z_$][\w$]*|'[^'\n]*'|"[^"\n]*")`;
  const match =
    new RegExp(`${key}\\s*:\\s*[A-Za-z_$][\\w$]*\\s*=\\s*$`, "u").exec(before) ??
    new RegExp(`${key}\\s*[:=]\\s*$`, "u").exec(before);
  return match?.[1]?.replace(/^['"]|['"]$/gu, "") ?? null;
}

/**
 * Whether any same-file type can declare `name`: every interface body, type
 * alias, `defineProps` type argument and `generic` attribute is scanned for a
 * `name:` or `name?:` member. Regions over-extend rather than stop early.
 */
function declaresPropName(source: string, regions: Range[], name: string): boolean {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const declaration = new RegExp(
    `(?:^|[^\\w$.'"])(?:${escaped}|'${escaped}'|"${escaped}")\\s*\\??\\s*:`,
    "u",
  );
  return regions.some((region) => declaration.test(source.slice(region.start, region.end)));
}

export function typeRegions(
  source: string,
  scripts: Range[],
  includes: (typeName: string) => boolean = () => true,
): Range[] {
  const regions: Range[] = propsCalls(source).map((call) => call.typeArgument);
  for (const match of source.matchAll(/<script\b[^>]*\bgeneric\s*=\s*("[^"]*"|'[^']*')/giu))
    regions.push({ start: match.index ?? 0, end: (match.index ?? 0) + match[0].length });
  for (const script of scripts) {
    const text = source.slice(script.start, script.end);
    for (const match of text.matchAll(/\binterface\s+([A-Za-z_$][\w$]*)[^{]*\{/gu)) {
      if (!includes(match[1] ?? "")) continue;
      const open = script.start + (match.index ?? 0) + match[0].length - 1;
      regions.push({ start: open, end: matchingBracket(source, open) ?? script.end });
    }
    for (const match of text.matchAll(/\btype\s+([A-Za-z_$][\w$]*)(?:\s*<[^=]*?>)?\s*=/gu)) {
      if (!includes(match[1] ?? "")) continue;
      const start = script.start + (match.index ?? 0) + match[0].length;
      regions.push({ start, end: aliasEnd(source, start, script.end) });
    }
  }
  return regions;
}

/** The end of a type alias: a depth-0 `;`, or a depth-0 line break between complete operands. */
function aliasEnd(source: string, start: number, limit: number): number {
  let depth = 0;
  for (let index = start; index < limit; index++) {
    const character = source[index] ?? "";
    if (character === "'" || character === '"' || character === "`")
      index = stringEnd(source, index);
    else if ("({[<".includes(character)) depth++;
    else if (")}]".includes(character) || (character === ">" && source[index - 1] !== "="))
      depth = Math.max(0, depth - 1);
    else if (depth === 0 && character === ";") return index;
    else if (depth === 0 && character === "\n") {
      const before = source.slice(start, index).trimEnd();
      const after = source.slice(index + 1, limit).trimStart();
      if (
        before.length > 0 &&
        !/[|&=,?:<(]$/u.test(before) &&
        !/^(?:[|&?:.]|extends\b)/u.test(after)
      )
        return index;
    }
  }
  return limit;
}

function importBindings(source: string, scripts: Range[]): Set<string> {
  const bindings = new Set<string>();
  for (const script of scripts) {
    const text = source.slice(script.start, script.end);
    for (const match of text.matchAll(/\bimport\s+(?:type\s+)?([^'";]*?)\s+from\s*['"]/gu)) {
      const clause = match[1] ?? "";
      const named = /\{([^}]*)\}/u.exec(clause)?.[1] ?? "";
      for (const part of named.split(",")) {
        const local = part
          .trim()
          .replace(/^type\s+/u, "")
          .split(/\s+as\s+/u)
          .pop()
          ?.trim();
        if (local) bindings.add(local);
      }
      const rest = clause.replace(/\{[^}]*\}/u, "");
      const namespace = /\*\s*as\s+([A-Za-z_$][\w$]*)/u.exec(rest)?.[1];
      if (namespace) bindings.add(namespace);
      const fallback = /^\s*([A-Za-z_$][\w$]*)/u.exec(rest.replace(/\*\s*as\s+[\w$]+/u, ""))?.[1];
      if (fallback && fallback !== "type") bindings.add(fallback);
    }
  }
  return bindings;
}

/** Follow local aliases and interface heritage from the type argument to an import. */
export function reachesImport(source: string, scripts: Range[], typeArgument: string): boolean {
  const imports = importBindings(source, scripts);
  return [...typeClosure(source, typeArgument)].some((name) => imports.has(name));
}

/** Every name the type argument reaches through local aliases and interface heritage. */
export function typeClosure(source: string, typeArgument: string): Set<string> {
  const pending = [...typeArgument.matchAll(identifier)].map((match) => match[0]);
  const seen = new Set<string>();
  while (pending.length > 0 && seen.size < maximumVisitedTypes) {
    const name = pending.pop();
    if (name === undefined || seen.has(name)) continue;
    seen.add(name);
    const escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    const heritage = new RegExp(
      `\\binterface\\s+${escaped}\\b(?:\\s*<[^>{}]*>)?\\s*([^{}]*)\\{`,
      "u",
    ).exec(source)?.[1];
    const alias = new RegExp(
      `\\btype\\s+${escaped}\\b(?:\\s*<[^;={}]*>)?\\s*=\\s*([^;]+)`,
      "u",
    ).exec(source)?.[1];
    for (const match of `${heritage ?? ""} ${alias ?? ""}`.matchAll(identifier))
      if (!seen.has(match[0])) pending.push(match[0]);
  }
  return seen;
}

function skipSpace(source: string, index: number): number {
  while (index < source.length && /\s/u.test(source[index] ?? "")) index++;
  return index;
}

/** The `>` closing a type argument list; `=>` arrows do not close it. */
export function matchingAngle(source: string, open: number): number | null {
  let depth = 1;
  for (let index = open; index < source.length; index++) {
    const character = source[index];
    if (character === "'" || character === '"' || character === "`")
      index = stringEnd(source, index);
    else if (character === "<") depth++;
    else if (character === ">" && source[index - 1] !== "=" && --depth === 0) return index;
  }
  return null;
}

function matchingBracket(source: string, open: number): number | null {
  const pairs: Record<string, string> = { "(": ")", "{": "}", "[": "]" };
  const stack: string[] = [];
  for (let index = open; index < source.length; index++) {
    const character = source[index] ?? "";
    if (character === "'" || character === '"' || character === "`")
      index = stringEnd(source, index);
    else if (character in pairs) stack.push(pairs[character] ?? "");
    else if (character === stack.at(-1)) {
      stack.pop();
      if (stack.length === 0) return index;
    }
  }
  return null;
}

/** The closing quote of a string literal; template literals are skipped whole. */
function stringEnd(source: string, open: number): number {
  const quote = source[open];
  for (let index = open + 1; index < source.length; index++) {
    if (source[index] === "\\") index++;
    else if (source[index] === quote) return index;
  }
  return source.length;
}
