import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { replaceReadmeVersionReferences } from "../../tools/support/release/release/readmes.ts";

const stampReadme = (file: string) =>
  replaceReadmeVersionReferences(fs.readFileSync(file, "utf8"), "0.425.0", "0.425.1");

for (const newline of ["\n", "\r\n"]) {
  test(`release preserves measured versions while updating install examples (${JSON.stringify(newline)})`, () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "release-readme-"));
    const file = path.join(directory, "README.md");
    const snapshot = [
      "<!-- benchmark:readme:start -->",
      "Versions: vize: `vize 0.425.0` · vue: `3.6.0`.",
      "| Typecheck | 0.425.0 | 2.03s |",
      "<!-- benchmark:readme:end -->",
    ].join(newline);
    const before = `Install vize@0.425.0${newline}${snapshot}${newline}npx vize@0.425.0 --version${newline}`;
    fs.writeFileSync(file, before);
    try {
      const result = stampReadme(file);
      assert.equal(
        result,
        `Install vize@0.425.1${newline}${snapshot}${newline}npx vize@0.425.1 --version${newline}`,
      );
      fs.writeFileSync(file, result);
      const repeated = stampReadme(file);
      assert.equal(repeated, result);
      fs.writeFileSync(file, "Install vize@0.425.0");
      const plain = stampReadme(file);
      assert.equal(plain, "Install vize@0.425.1");
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}
