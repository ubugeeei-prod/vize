import { lexicalView } from "../../../tools/support/levels/move-host-runtime.ts";
import {
  pathHostCallers,
  pathHostFunctions,
} from "../../../tools/support/compat/levels/path-host-callers.ts";

/** Admit only the three existing path APIs at their complete physical host callers. */
export function withoutPathHostReferences(source: string, file: string): string {
  const expected = pathHostCallers[file];
  if (!expected) return source;
  const view = lexicalView(source);
  const matches = pathHostFunctions.map((name) => [
    ...view.matchAll(
      new RegExp(`(?<![\\p{ID_Continue}])vize_carton::path::${name}(?![\\p{ID_Continue}])`, "gu"),
    ),
  ]);
  if (
    matches.some((found, index) => found.length !== expected[index]) ||
    [...view.matchAll(/(?<![\p{ID_Continue}])vize_carton\s*::\s*path(?![\p{ID_Continue}])/gu)]
      .length !== matches.flat().length
  )
    return source;
  for (const match of matches.flat().sort((a, b) => b.index - a.index)) {
    source =
      source.slice(0, match.index) +
      "host_path_runtime" +
      source.slice(match.index + match[0].length);
  }
  return source;
}
