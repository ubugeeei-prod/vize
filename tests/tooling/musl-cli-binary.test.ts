import assert from "node:assert/strict";
import { test } from "node:test";

import { glibcRequirementLines } from "../../tools/commands/ci/github/verify-musl-cli-binary.ts";
import { readRepoFile } from "./support/github-workflows.ts";

test("musl verifier deduplicates and sorts glibc requirements", () => {
  assert.deepEqual(
    glibcRequirementLines(`
      0x0010:   Name: GLIBC_2.17  Flags: none  Version: 5
      0x0020:   Name: GLIBC_2.36  Flags: none  Version: 6
      0x0010:   Name: GLIBC_2.17  Flags: none  Version: 5
    `),
    [
      "      0x0010:   Name: GLIBC_2.17  Flags: none  Version: 5",
      "      0x0020:   Name: GLIBC_2.36  Flags: none  Version: 6",
    ],
  );
});

test("musl verifier reads compact ELF version metadata", () => {
  const verifier = readRepoFile("tools", "commands", "ci", "github", "verify-musl-cli-binary.ts");

  assert.match(verifier, /commandOutput\("readelf", \["--version-info", binary\]\)/u);
  assert.doesNotMatch(verifier, /commandOutput\("strings"/u);
});
