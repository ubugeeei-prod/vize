import assert from "node:assert/strict";
import { test } from "node:test";

import {
  glibcVersionsAbove,
  highestGlibcVersion,
  parseGlibcVersion,
  parseGlibcVersions,
} from "../../tools/github/verify-glibc-symbols.mjs";

test("glibc symbol parser deduplicates and sorts required versions", () => {
  const versions = parseGlibcVersions(`
    0x0010:   Name: GLIBC_2.2.5  Flags: none  Version: 7
    0x0020:   Name: GLIBC_2.36   Flags: none  Version: 6
    0x0030:   Name: GLIBC_2.17   Flags: none  Version: 5
    0x0040:   Name: GLIBC_2.36   Flags: none  Version: 6
  `);

  assert.deepEqual(
    versions.map((version) => version.text),
    ["2.2.5", "2.17", "2.36"],
  );
  assert.equal(highestGlibcVersion(versions)?.text, "2.36");
});

test("glibc verifier flags native binaries newer than the Debian bookworm ceiling", () => {
  const versions = parseGlibcVersions(`
    0x0010:   Name: GLIBC_2.2.5  Flags: none  Version: 7
    0x0020:   Name: GLIBC_2.39   Flags: none  Version: 6
  `);

  assert.deepEqual(
    glibcVersionsAbove(versions, parseGlibcVersion("2.36")).map((version) => version.text),
    ["2.39"],
  );
});

test("glibc verifier accepts binaries at the Debian bookworm ceiling", () => {
  const versions = parseGlibcVersions(`
    0x0010:   Name: GLIBC_2.2.5  Flags: none  Version: 7
    0x0020:   Name: GLIBC_2.36   Flags: none  Version: 6
  `);

  assert.deepEqual(glibcVersionsAbove(versions, parseGlibcVersion("2.36")), []);
});
