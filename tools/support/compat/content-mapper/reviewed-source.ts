/** Where a custody record reads the bytes of a source it reviewed. */
import { existsSync } from "node:fs";

const repository = new URL("../../../../", import.meta.url);

/**
 * The source a custody record reviewed, under `root`. A source that has since
 * moved on is read from its frozen copy under `tests/_fixtures/byte-custody`.
 */
export function reviewedSource(path: string, root: URL = repository): URL {
  const frozen = new URL(`tests/_fixtures/byte-custody/${path}`, root);
  return existsSync(frozen) ? frozen : new URL(path, root);
}
