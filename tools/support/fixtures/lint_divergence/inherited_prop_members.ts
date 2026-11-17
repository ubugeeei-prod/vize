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

/** Type names reached from `text` through same-file aliases and interface heritage. */
function closure(source: string, text: string): Set<string> {
  const pending = [...text.matchAll(identifier)].map((match) => match[0]);
  const seen = new Set<string>();
  while (pending.length > 0 && seen.size < maximumVisitedTypes) {
    const name = pending.pop();
    if (name === undefined || seen.has(name)) continue;
    seen.add(name);
    const escaped = name.replace(/\$/gu, "\\$");
    const heritage = new RegExp(`\\binterface\\s+${escaped}\\b[^{}]*\\{`, "u").exec(source)?.[0];
    const alias = new RegExp(`\\btype\\s+${escaped}\\b[^=]*=([^;]+)`, "u").exec(source)?.[1];
    for (const match of `${heritage ?? ""} ${alias ?? ""}`.matchAll(identifier))
      if (!seen.has(match[0])) pending.push(match[0]);
  }
  return seen;
}
