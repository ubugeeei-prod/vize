// Events that only an imported emits type declares.
//
// Like imported props, a `defineEmits<T>()` event that T receives from
// another module is unknown to the eslint-plugin-vue baseline, which runs
// without type-aware parser services. Patina resolves the module, so its
// `no-shadow-native-events` finding on the type argument is accepted here
// only when the type reaches an import binding and no same-file type it
// reaches spells the event.

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

/** A native event Patina reports on a `defineEmits<T>()` type argument. */
export function nativeEventFromImportedContract(
  message: string,
  start: number,
  end: number,
  source: string,
): boolean {
  const name = /`([^`]+)`/u.exec(message)?.[1];
  if (name === undefined) return false;
  const scripts = scriptBlocks(source);
  for (const match of source.matchAll(/\bdefineEmits\s*</gu)) {
    const open = (match.index ?? 0) + match[0].length;
    const close = matchingAngle(source, open);
    if (close === null || start < open || close < end) continue;
    const argument = source.slice(open, close);
    if (!reachesImport(source, scripts, argument)) return false;
    const reached = typeClosure(source, argument);
    const regions = [
      { start: open, end: close },
      ...typeRegions(source, scripts, (typeName) => reached.has(typeName)),
    ];
    const escaped = escapeName(name);
    // A member key (`click: []`) or any string spelling (`(e: 'click')`, `Pick<_, 'click'>`).
    const spelled = new RegExp(
      `(?:^|[^\\w$.'"])${escaped}\\s*\\??\\s*:|['"\`]${escaped}['"\`]`,
      "u",
    );
    return !regions.some((region) => spelled.test(source.slice(region.start, region.end)));
  }
  return false;
}
