import remapping from "@jridgewell/remapping";
import convertSourceMap from "convert-source-map";
import type { SourceMapInput } from "@jridgewell/remapping";

export function composeInputMap(nativeMap: string | null | undefined, printedMap: object | null) {
  if (!nativeMap || !printedMap) {
    throw new Error("The native JSX compiler did not provide the requested original-source map");
  }
  const parsed: unknown = JSON.parse(nativeMap);
  if (
    typeof parsed !== "object" ||
    parsed === null ||
    !("version" in parsed) ||
    parsed.version !== 3 ||
    !("mappings" in parsed) ||
    typeof parsed.mappings !== "string" ||
    !("sources" in parsed) ||
    !Array.isArray(parsed.sources)
  ) {
    throw new Error("The native JSX compiler returned an invalid source map");
  }
  // Babel's first generation already composes the caller's upstream map. The
  // last map can have multiple authored sources; no identity map is invented.
  const composed = remapping([parsed as SourceMapInput, printedMap as SourceMapInput], () => null);
  return convertSourceMap.fromObject(composed);
}
