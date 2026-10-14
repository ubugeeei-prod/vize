import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const archive = "tests/_fixtures/differential/config/vue-namespace/source-custody/";
// The historical Vite vectors are read from archived copies so that editing the
// live Vite tests or the glyph fixture never changes these formatter inputs.
const originalFamily = {
  "npm/builder/vite/src/vite-plus/import-sorting.test.ts": {
    archive: archive + "original-vite-plus-import-sorting.test.ts.fixture",
    bytes: 3809,
    sha256: "02b2910051683dca5108a47a622db63dc086f5a0a9b599b20ab33e043473832c",
  },
  "npm/builder/vite/src/vite-plus/config.test.ts": {
    archive: archive + "original-vite-plus-config.test.ts.fixture",
    bytes: 9327,
    sha256: "82570d354aa7cfb9300bca05be49e7bd451694beea019b635cac1ccb7c8056a4",
  },
  "crates/vize_glyph/tests/fixtures/sort-imports/UserCard.vue": {
    archive: archive + "original-sort-imports-user-card.vue.fixture",
    bytes: 363,
    sha256: "ae86278aa0a0b247d9e027b703699f02cf370ea37da9dd96368adf3ff334b106",
  },
};
export const originalViteArchives = Object.fromEntries(
  Object.entries(originalFamily).map(([file, { archive: copy }]) => [file, copy]),
);
const hash = (raw) => createHash("sha256").update(raw).digest("hex");

/** Return the archived historical Vite input family, keyed by its original paths. */
export function originalViteSourceFamily(root) {
  return Object.fromEntries(
    Object.entries(originalFamily).map(([file, expected]) => {
      const target = path.join(root, expected.archive);
      const mode = fs.lstatSync(target);
      assert(mode.isFile(), `historical Vite vector changed: ${file} archive is not a file`);
      const raw = fs.readFileSync(target);
      assert(
        raw.length === expected.bytes && hash(raw) === expected.sha256,
        `historical Vite vector changed: ${file}`,
      );
      return [file, raw.toString()];
    }),
  );
}
