/**
 * eslint-plugin-vue reads the members of the `defineProps<T>()` type and of
 * the unions, intersections and aliases T spells, but it never follows an
 * interface `extends` clause. Patina does, so a member that the props reach
 * only through heritage is reported by Patina alone.
 */
export const inheritedPropMemberReason =
  "Patina reads props a defineProps type inherits through a same-file interface `extends` clause; eslint-plugin-vue never follows interface heritage, so it does not see this member.";

const identifier = /[A-Za-z_$][\w$]*/gu;
const maximumVisitedTypes = 32;

/**
 * The member at `offset` sits directly in the body of a same-file interface
 * that every other mention names in an `extends` clause, and an interface
 * extending it is reached from the defineProps type argument.
 */
export function inheritedPropMember(offset: number, source: string): boolean {
  const header = /\binterface\s+([A-Za-z_$][\w$]*)[^{}]*\{[^{}]*$/u.exec(source.slice(0, offset));
  const name = header?.[1];
  const argument = /\bdefineProps\s*<([^()]*?)>\s*\(/u.exec(source)?.[1];
  if (name === undefined || argument === undefined) return false;
  const word = new RegExp(`(?<![\\w$])${name.replace(/\$/gu, "\\$")}(?![\\w$])`, "gu");
  const heirs = new Set<string>();
  for (const mention of source.matchAll(word)) {
    const at = mention.index ?? 0;
    if (/\binterface\s+$/u.test(source.slice(Math.max(0, at - 20), at))) {
      if (at !== (header?.index ?? -1) + (header?.[0].indexOf(name) ?? 0)) return false;
      continue;
    }
    const heir = /\binterface\s+([A-Za-z_$][\w$]*)[^{}]*\bextends\b[^{}]*$/u.exec(
      source.slice(Math.max(0, at - 500), at),
    );
    if (heir?.[1] === undefined) return false;
    heirs.add(heir[1]);
  }
  const reached = closure(source, argument);
  return [...heirs].some((heir) => reached.has(heir));
}

/** Type names reached from `text` through same-file aliases and, unless off, interface heritage. */
function closure(source: string, text: string, followHeritage = true): Set<string> {
  const pending = [...text.matchAll(identifier)].map((match) => match[0]);
  const seen = new Set<string>();
  while (pending.length > 0 && seen.size < maximumVisitedTypes) {
    const name = pending.pop();
    if (name === undefined || seen.has(name)) continue;
    seen.add(name);
    const escaped = name.replace(/\$/gu, "\\$");
    const heritage = followHeritage
      ? new RegExp(`\\binterface\\s+${escaped}\\b[^{}]*\\{`, "u").exec(source)?.[0]
      : undefined;
    const alias = new RegExp(`\\btype\\s+${escaped}\\b[^=]*=([^;]+)`, "u").exec(source)?.[1];
    for (const match of `${heritage ?? ""} ${alias ?? ""}`.matchAll(identifier))
      if (!seen.has(match[0])) pending.push(match[0]);
  }
  return seen;
}

export const heritageOnlyPropReason =
  "The shadowed prop is declared only in a same-file interface the defineProps type extends; Patina reads that heritage, while eslint-plugin-vue never follows an interface `extends` clause and does not know the prop.";

/**
 * Prop `name` is a member of a same-file interface the defineProps type
 * reaches through `extends`, and no type eslint-plugin-vue reads (the
 * argument, the aliases it spells and their interfaces) names it. An
 * `Omit<T, 'name'>` heritage item is not followed, nor a `Pick` without it.
 */
export function heritageOnlyProp(name: string, source: string): boolean {
  const argument = /\bdefineProps\s*<([^()]*?)>\s*\(/u.exec(source)?.[1];
  if (argument === undefined) return false;
  const escaped = name.replace(/[$.]/gu, "\\$&");
  const member = new RegExp(
    `(?:^|[^\\w$.'"])(?:${escaped}|'${escaped}'|"${escaped}")\\s*\\??\\s*:`,
    "u",
  );
  const quoted = new RegExp(`['"]${escaped}['"]`, "u");
  if (member.test(argument)) return false;
  const read = closure(source, argument, false);
  if ([...read].some((type) => member.test(`${body(source, type)} ${alias(source, type) ?? ""}`)))
    return false;
  const pending = [...read];
  const seen = new Set<string>();
  while (pending.length > 0 && seen.size < maximumVisitedTypes) {
    const type = pending.pop();
    if (type === undefined || seen.has(type)) continue;
    seen.add(type);
    if (!read.has(type) && member.test(body(source, type))) return true;
    for (const item of heritageItems(source, type)) {
      const utility = /^(Omit|Pick)\s*</u.exec(item)?.[1];
      if ((utility === "Omit" && quoted.test(item)) || (utility === "Pick" && !quoted.test(item)))
        continue;
      for (const match of item.matchAll(identifier)) pending.push(match[0]);
    }
  }
  return false;
}

function body(source: string, type: string): string {
  const header = new RegExp(`\\binterface\\s+${type.replace(/\$/gu, "\\$")}\\b[^{}]*\\{`, "u").exec(
    source,
  );
  if (header === null) return "";
  const open = header.index + header[0].length - 1;
  let depth = 0;
  for (let index = open; index < source.length; index++) {
    if (source[index] === "{") depth++;
    else if (source[index] === "}" && --depth === 0) return source.slice(open, index + 1);
  }
  return source.slice(open);
}

function alias(source: string, type: string): string | undefined {
  return new RegExp(`\\btype\\s+${type.replace(/\$/gu, "\\$")}\\b[^=]*=([^;]+)`, "u").exec(
    source,
  )?.[1];
}

/** The top-level items of `interface type extends ...`, comments removed. */
function heritageItems(source: string, type: string): string[] {
  const header = new RegExp(
    `\\binterface\\s+${type.replace(/\$/gu, "\\$")}\\b([^{}]*)\\{`,
    "u",
  ).exec(source)?.[1];
  const clause = /\bextends\b([\s\S]*)$/u.exec(
    (header ?? "").replace(/\/\*[\s\S]*?\*\/|\/\/[^\n]*/gu, " "),
  )?.[1];
  if (clause === undefined) return [];
  const items: string[] = [];
  let depth = 0,
    start = 0;
  for (let index = 0; index < clause.length; index++) {
    const character = clause[index];
    if (character === "<" || character === "(" || character === "[") depth++;
    else if (character === ">" || character === ")" || character === "]") depth--;
    else if (character === "," && depth === 0) {
      items.push(clause.slice(start, index).trim());
      start = index + 1;
    }
  }
  items.push(clause.slice(start).trim());
  return items;
}
