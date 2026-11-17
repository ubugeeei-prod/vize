// Events and slots that only an imported macro type declares.
//
// Like imported props, a `defineEmits<T>()` event or `defineSlots<T>()` slot
// that T receives from another module is unknown to the eslint-plugin-vue
// baseline, which runs without type-aware parser services. Patina resolves
// the module. A difference is accepted here only when the type argument
// reaches an import binding and no same-file type it reaches spells the
// name as a member key or string.

import {
  escapeName,
  matchingAngle,
  reachesImport,
  scriptBlocks,
  typeClosure,
  typeRegions,
} from "./imported_prop_contract.ts";

export const importedEmitContractReason =
  "Patina resolves events declared by an emits type imported from another module; the eslint-plugin-vue baseline runs without type-aware parser services, so those events are unknown to it.";
export const importedSlotContractReason =
  "Patina resolves slots declared by a slots type imported from another module; the eslint-plugin-vue baseline runs without type-aware parser services, so it reports those slots as undefined.";

interface TypeArgument {
  readonly start: number;
  readonly end: number;
}

/**
 * An event Patina reports on a `defineEmits<T>()` type argument: a native
 * event it shadows, or one it declares but never emits.
 */
export function emitFindingFromImportedContract(
  message: string,
  start: number,
  end: number,
  source: string,
): boolean {
  const name = (/^\[vize:vue\/no-shadow-native-events\] .*`([^`]+)`$/u.exec(message) ??
    /^\[vize:script\/no-unused-emit-declarations\] The '([^']+)' event is declared but never emitted\.$/u.exec(
      message,
    ))?.[1];
  if (name === undefined) return false;
  const argument = typeArguments(source, "defineEmits").find(
    (candidate) => candidate.start <= start && end <= candidate.end,
  );
  return argument !== undefined && onlyImportedDeclares(source, argument, name);
}

/** An upstream "Slots must be explicitly defined." on a `<slot>` an imported slots type declares. */
export function undeclaredSlotFromImportedContract(offset: number, source: string): boolean {
  if (!source.startsWith("<slot", offset) || !/[\s/>]/u.test(source[offset + 5] ?? ""))
    return false;
  const tag = source.slice(offset, source.indexOf(">", offset) + 1);
  // A bound name could be anything; only a static or absent name is checked,
  // and a bound literal (`:name="\`month-overlay-value\`"`) is static.
  const literal = /\s(?::|v-bind:)name\s*=\s*"\s*(?:`([^`$]*)`|'([^']*)')\s*"/u.exec(tag);
  if (literal === null && /\s(?::|v-bind:)name\b/u.test(tag)) return false;
  const name = literal ?? /\sname\s*=\s*(?:"([^"]*)"|'([^']*)')/u.exec(tag);
  const slot = name === null ? "default" : (name[1] ?? name[2] ?? "");
  const [argument, ...others] = typeArguments(source, "defineSlots");
  return (
    argument !== undefined &&
    others.length === 0 &&
    slot !== "" &&
    onlyImportedDeclares(source, argument, slot)
  );
}

function typeArguments(source: string, macro: "defineEmits" | "defineSlots"): TypeArgument[] {
  const found: TypeArgument[] = [];
  for (const match of source.matchAll(new RegExp(`\\b${macro}\\s*<`, "gu"))) {
    const start = (match.index ?? 0) + match[0].length;
    const end = matchingAngle(source, start);
    if (end !== null) found.push({ start, end });
  }
  return found;
}

/** The type argument reaches an import and no same-file type it reaches spells `name`. */
function onlyImportedDeclares(source: string, argument: TypeArgument, name: string): boolean {
  const scripts = scriptBlocks(source);
  const text = source.slice(argument.start, argument.end);
  if (!reachesImport(source, scripts, text)) return false;
  const reached = typeClosure(source, text);
  // A prop such as `icon?: IconName` names no slot or event, so the props
  // argument is not a declaring region here.
  const regions = [
    argument,
    ...typeRegions(source, scripts, (typeName) => reached.has(typeName), false),
  ];
  const escaped = escapeName(name);
  // A member key (`click: []`, `default?(props)`) or any string spelling.
  const spelled = new RegExp(
    `(?:^|[^\\w$.'"])${escaped}\\s*\\??\\s*[:(]|['"\`]${escaped}['"\`]`,
    "u",
  );
  // A lone `Pick<Slots, ...>` declares only its keys.
  const lonePick =
    /^\s*Pick\s*<[\s\S]*>\s*$/u.test(text) && !/[&|]/u.test(withoutUtilityKeys(text));
  if (lonePick && !new RegExp(`['"\`]${escaped}['"\`]`, "u").test(text)) return false;
  return !regions.some((region) =>
    spelled.test(withoutUtilityKeys(source.slice(region.start, region.end))),
  );
}

/**
 * `Pick<Slots, 'item'>` quotes member names without declaring them: the
 * keys select from the imported type. Pick keys are blanked so only a
 * declaring spelling counts. `Omit` keys stay, so an omitted name is never
 * classified.
 */
function withoutUtilityKeys(text: string): string {
  let result = text;
  for (const match of text.matchAll(/\bPick\s*</gu)) {
    const open = (match.index ?? 0) + match[0].length;
    let depth = 0,
      comma = -1;
    for (let index = open; index < text.length; index++) {
      const character = text[index];
      if (character === "<" || character === "(" || character === "{" || character === "[") depth++;
      else if (character === ")" || character === "}" || character === "]") depth--;
      else if (character === ">") {
        if (depth === 0) {
          if (comma >= 0)
            result =
              result.slice(0, comma + 1) + " ".repeat(index - comma - 1) + result.slice(index);
          break;
        }
        depth--;
      } else if (character === "," && depth === 0) comma = index;
    }
  }
  return result;
}
