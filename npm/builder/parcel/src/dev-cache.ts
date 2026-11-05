import { Reporter } from "@parcel/plugin";
import {
  advanceDevBundleMtime,
  type DevCacheRecord,
  type DevOutputFile,
} from "./dev-cache-policy.ts";

const records = new Map<string, DevCacheRecord>();

/**
 * Run before `@parcel/reporter-dev-server`. HMR is broadcast while requests are
 * still queued, and the dev server releases them on this same buildSuccess.
 */
export default new Reporter({
  async report({ event, options }) {
    if (event.type !== "buildSuccess" || options.mode === "production" || !options.hmrOptions) {
      return;
    }
    const output = options.outputFS as DevOutputFile;
    for (const bundle of event.bundleGraph.getBundles()) {
      if (bundle.bundleBehavior === "inline" || bundle.filePath.length === 0) continue;
      const previous = records.get(bundle.filePath);
      try {
        records.set(
          bundle.filePath,
          await advanceDevBundleMtime(output, bundle.filePath, previous),
        );
      } catch (error) {
        if (
          typeof error === "object" &&
          error !== null &&
          "code" in error &&
          error.code === "ENOENT"
        ) {
          continue;
        }
        throw error;
      }
    }
  },
});
