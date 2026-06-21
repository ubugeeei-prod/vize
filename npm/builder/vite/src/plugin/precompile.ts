import {
  chunkVitePrecompileFiles,
  diffVitePrecompileFiles,
  hasVitePrecompileFileMetadataChanged,
  normalizeVitePrecompileBatchSize,
  type VitePrecompileFileMetadataEntryNapi,
} from "@vizejs/native";

export const DEFAULT_PRECOMPILE_BATCH_SIZE = 128;
export const DEFAULT_PRECOMPILE_BATCH_MAX_BYTES = 32 * 1024 * 1024;

export const DEFAULT_PRECOMPILE_IGNORE_PATTERNS = [
  "node_modules/**",
  "dist/**",
  ".git/**",
  ".nuxt/**",
  ".output/**",
  ".nitro/**",
  "coverage/**",
];

export interface PrecompileFileMetadata {
  mtimeMs: number;
  size: number;
}

export interface PrecompileDiff {
  changedFiles: string[];
  deletedFiles: string[];
}

export interface PrecompileChunkOptions {
  maxBytes?: number;
  metadata?: ReadonlyMap<string, PrecompileFileMetadata>;
}

export function isPrecompileSfcPath(path: string): boolean {
  return path.endsWith(".vue");
}

export function hasFileMetadataChanged(
  previous: PrecompileFileMetadata | undefined,
  next: PrecompileFileMetadata,
): boolean {
  return hasVitePrecompileFileMetadataChanged(previous, next);
}

export function diffPrecompileFiles(
  files: readonly string[],
  currentMetadata: ReadonlyMap<string, PrecompileFileMetadata>,
  previousMetadata: ReadonlyMap<string, PrecompileFileMetadata>,
): PrecompileDiff {
  return diffVitePrecompileFiles(
    [...files],
    toNativePrecompileMetadataEntries(currentMetadata),
    toNativePrecompileMetadataEntries(previousMetadata),
  );
}

export function normalizePrecompileBatchSize(value: number | undefined): number {
  return normalizeVitePrecompileBatchSize(value);
}

export function chunkPrecompileFiles(
  files: readonly string[],
  batchSize: number,
  options: PrecompileChunkOptions = {},
): string[][] {
  return chunkVitePrecompileFiles([...files], batchSize, {
    maxBytes: options.maxBytes,
    metadata: options.metadata ? toNativePrecompileMetadataEntries(options.metadata) : undefined,
  });
}

function toNativePrecompileMetadataEntries(
  metadata: ReadonlyMap<string, PrecompileFileMetadata>,
): VitePrecompileFileMetadataEntryNapi[] {
  const entries: VitePrecompileFileMetadataEntryNapi[] = [];
  for (const [path, value] of metadata) {
    entries.push({ path, mtimeMs: value.mtimeMs, size: value.size });
  }
  return entries;
}
