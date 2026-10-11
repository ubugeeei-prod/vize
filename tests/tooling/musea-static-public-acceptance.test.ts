import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";
import {
  adaptDriver,
  digest,
  drivers,
} from "../../tools/support/release/musea_static_acceptance/fixtures.ts";
import {
  assertFiles,
  assertSuiteCensus,
} from "../../tools/support/release/musea_static_acceptance/custody.ts";

void test("public static adapter preserves all original runtime assertions and records explicit changes", () => {
  for (const [source, file] of drivers) {
    const original = fs.readFileSync(source, "utf8");
    const adapted = adaptDriver(file, original, "/tmp/public-consumer");
    let reversed = adapted.bytes;
    for (const replacement of [...adapted.replacements].reverse()) {
      assert.equal(reversed.split(replacement.to).length, 2);
      reversed = reversed.replace(replacement.to, replacement.from);
    }
    assert.equal(reversed, original);
    const assertions = (value: string) =>
      value.split("\n").filter((line) => /^\s*assert\./u.test(line));
    assert.deepEqual(assertions(adapted.bytes), assertions(original));
  }
  const source = fs.readFileSync(
    "npm/builder/vite-musea/src/inline-art-static.fixtures.ts",
    "utf8",
  );
  assert.throws(
    () =>
      adaptDriver(
        "inline-art-static.fixtures.ts",
        source.replace('import vize from "../../vite/src/index.ts";', ""),
        "/tmp/public-consumer",
      ),
    /unique adapter source/u,
  );
});

void test("terminal suite census rejects omitted, skipped and duplicate summaries", () => {
  const complete = "# tests 3\n# pass 3\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n";
  assertSuiteCensus(complete);
  for (const invalid of [
    complete.replace("# tests 3\n", ""),
    complete.replace("# skipped 0", "# skipped 1"),
    complete + "# pass 3\n",
  ])
    assert.throws(() => assertSuiteCensus(invalid));
});

void test("archive custody rejects a changed fixture and a provider file redirected outside the consumer", (t) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(tmpdir(), "musea-static-custody-law-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.writeFileSync(path.join(root, "provider.node"), "original");
  const record = [{ file: "provider.node", sha256: digest("original") }];
  assertFiles(root, record);
  fs.writeFileSync(path.join(root, "provider.node"), "changed");
  assert.throws(() => assertFiles(root, record));
  fs.unlinkSync(path.join(root, "provider.node"));
  fs.symlinkSync(path.join(root, "elsewhere.node"), path.join(root, "provider.node"));
  fs.writeFileSync(path.join(root, "elsewhere.node"), "original");
  assert.throws(() => assertFiles(root, record));
});
