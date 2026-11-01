import { loadNative } from "../native-loader.js";
import { isTrustedSourcePath } from "../trusted-path.js";
import { shouldProcess } from "../utils.js";

interface RawArtInputConfig {
  root: string;
  include: string[];
  exclude: string[];
  scanRoots: readonly string[];
}

/** Only the active Musea instance may delegate raw Art to its virtual SFC route. */
export function createRawArtInputOwner(config: RawArtInputConfig) {
  const root = config.root;
  const include = [...config.include];
  const exclude = [...config.exclude];
  const scanRoots = [...config.scanRoots];
  return Object.freeze({
    ownsInput(file: string, source: string): boolean {
      if (
        !file.endsWith(".art.vue") ||
        !shouldProcess(file, include, exclude, root) ||
        !isTrustedSourcePath(root, scanRoots, file)
      ) {
        return false;
      }
      const native = loadNative();
      // Structural parsing remains tolerant of custom-only descriptors. It does
      // not invoke compileSfc or treat parseArt error text as an absence marker.
      const descriptor = native.parseSfc(source, { filename: file });
      if (!descriptor.customBlocks.some((block) => block.type === "art")) return false;
      // A malformed or empty Art file is not a successfully owned virtual input.
      // Parser/metadata errors propagate, so ownership cannot hide a failure.
      return native.parseArt(source, { filename: file }).variants.length > 0;
    },
  });
}
