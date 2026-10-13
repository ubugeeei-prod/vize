import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { runMoonScript } from "./_helpers/moonbit.ts";

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
      const result = runMoonScript("release", [
        "--print-readme-version-update",
        file,
        "0.425.0",
        "0.425.1",
      ]);
      assert.equal(result.status, 0, result.stderr);
      assert.equal(
        result.stdout,
        `Install vize@0.425.1${newline}${snapshot}${newline}npx vize@0.425.1 --version${newline}`,
      );
      fs.writeFileSync(file, result.stdout);
      const repeated = runMoonScript("release", [
        "--print-readme-version-update",
        file,
        "0.425.0",
        "0.425.1",
      ]);
      assert.equal(repeated.status, 0, repeated.stderr);
      assert.equal(repeated.stdout, result.stdout);
      fs.writeFileSync(file, "Install vize@0.425.0");
      const plain = runMoonScript("release", [
        "--print-readme-version-update",
        file,
        "0.425.0",
        "0.425.1",
      ]);
      assert.equal(plain.status, 0, plain.stderr);
      assert.equal(plain.stdout, "Install vize@0.425.1");
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}
