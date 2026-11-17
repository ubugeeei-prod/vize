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
  // A bound name could be anything; only a static or absent name is checked.
  if (/\s(?::|v-bind:)name\b/u.test(tag)) return false;
  const name = /\sname\s*=\s*(?:"([^"]*)"|'([^']*)')/u.exec(tag);
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
  const regions = [argument, ...typeRegions(source, scripts, (typeName) => reached.has(typeName))];
  const escaped = escapeName(name);
  // A member key (`click: []`, `default?(props)`) or any string spelling.
  const spelled = new RegExp(
    `(?:^|[^\\w$.'"])${escaped}\\s*\\??\\s*[:(]|['"\`]${escaped}['"\`]`,
    "u",
  );
  return !regions.some((region) => spelled.test(source.slice(region.start, region.end)));
}
