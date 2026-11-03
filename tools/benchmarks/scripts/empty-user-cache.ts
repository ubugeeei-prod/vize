/**
 * `vize check` keeps its generated project and tsgo's `--incremental` build
 * info in the user cache (`dirs::cache_dir`, which reads `XDG_CACHE_HOME` on
 * Linux). A second run over an unchanged project then replays the first run's
 * diagnostics instead of checking them, whatever its checker width. A timed or
 * compared run must therefore start from an empty cache. On macOS
 * `dirs::cache_dir` ignores `XDG_CACHE_HOME`, so this only isolates Linux runs,
 * which is where every published benchmark runs.
 */

import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

/** Run with a fresh, empty `XDG_CACHE_HOME`, removed afterwards. */
export function withEmptyUserCache<T>(run: (env: { XDG_CACHE_HOME: string }) => T): T {
  const cache = mkdtempSync(join(tmpdir(), "vize-user-cache-"));
  try {
    return run({ XDG_CACHE_HOME: cache });
  } finally {
    rmSync(cache, { recursive: true, force: true });
  }
}
