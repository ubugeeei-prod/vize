import type { VirtualTsMapping } from "../../wasm/types/analysis";

export function parseSourceMap(virtualTs: string): VirtualTsMapping[] {
  return Array.from(
    virtualTs.matchAll(/\/\/ @vize-map:\s*(\d+):(\d+)\s*->\s*(\d+):(\d+)/g),
    (match) => ({
      genStart: Number(match[1]),
      genEnd: Number(match[2]),
      srcStart: Number(match[3]),
      srcEnd: Number(match[4]),
    }),
  );
}

export function mapGeneratedRange(
  start: number,
  end: number,
  mappings: readonly VirtualTsMapping[],
) {
  const candidates = mappings
    .flatMap((mapping) => [...(mapping.subSpans ?? []), mapping])
    .filter((mapping) => start >= mapping.genStart && start < mapping.genEnd)
    .sort((a, b) => a.genEnd - a.genStart - (b.genEnd - b.genStart));
  const mapping = candidates[0];
  if (!mapping) return null;
  if (mapping.genEnd - mapping.genStart !== mapping.srcEnd - mapping.srcStart) {
    return { start: mapping.srcStart, end: mapping.srcEnd };
  }
  const mappedStart = mapping.srcStart + start - mapping.genStart;
  return {
    start: mappedStart,
    end: Math.min(mapping.srcEnd, mappedStart + Math.max(0, end - start)),
  };
}
