import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  ariaDeclarationDigests,
  copyAriaDeclarations,
  sourceAriaDeclarationFiles,
} from "../../tools/support/compat/content-mapper/aria-source-package.ts";
import { repoRoot } from "./_helpers/tool-commands.ts";

const source = path.join(repoRoot, "npm/cli");
const manifest = JSON.parse(fs.readFileSync(path.join(source, "package.json"), "utf8"));
const declared = manifest.files.filter((file: string) => /\.d\.(?:ts|mts|cts)$/u.test(file));

test("source ARIA consumers copy every declared physical SDK file without altering its bytes", () => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-aria-sdk-"));
  try {
    const files = sourceAriaDeclarationFiles(manifest);
    assert.deepEqual(files, declared);
    assert.equal(files.length, 13);
    copyAriaDeclarations(source, scratch, files);
    for (const file of declared) {
      assert.deepEqual(
        fs.readFileSync(path.join(scratch, file)),
        fs.readFileSync(path.join(source, file)),
      );
    }
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
});

test("every missing declared SDK file rejects the source package before writing a partial copy", () => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-aria-sdk-missing-"));
  try {
    const input = path.join(scratch, "input");
    copyAriaDeclarations(source, input, declared);
    for (const [index, file] of declared.entries()) {
      const original = fs.readFileSync(path.join(input, file));
      fs.unlinkSync(path.join(input, file));
      const output = path.join(scratch, `missing-${index}`);
      assert.throws(
        () => copyAriaDeclarations(input, output, sourceAriaDeclarationFiles(manifest)),
        { code: "ENOENT" },
      );
      assert.equal(fs.existsSync(output), false);
      fs.writeFileSync(path.join(input, file), original);
    }
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
});

test("the source SDK exports the checker module included in its copied declarations", () => {
  assert.equal(manifest.exports["./virtual-ts-checker"].types, "./virtual-ts-checker.d.ts");
  assert.ok(sourceAriaDeclarationFiles(manifest).includes("virtual-ts-checker.d.ts"));
});

test("the declaration receipt detects changes to every shipped SDK declaration", () => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-aria-sdk-digests-"));
  try {
    const files = sourceAriaDeclarationFiles(manifest);
    copyAriaDeclarations(source, scratch, files);
    const expected = files.map((file) => ({
      file,
      sha256: createHash("sha256")
        .update(fs.readFileSync(path.join(source, file)))
        .digest("hex"),
    }));
    assert.deepEqual(ariaDeclarationDigests(scratch, files), expected);
    for (const [index, file] of files.entries()) {
      const target = path.join(scratch, file);
      const original = fs.readFileSync(target);
      const changed = Buffer.concat([original, Buffer.from("\n// changed declaration\n")]);
      fs.writeFileSync(target, changed);
      assert.deepEqual(
        ariaDeclarationDigests(scratch, files),
        expected.map((entry, position) =>
          position === index
            ? { file, sha256: createHash("sha256").update(changed).digest("hex") }
            : entry,
        ),
      );
      fs.writeFileSync(target, original);
    }
    assert.deepEqual(ariaDeclarationDigests(scratch, files), expected);
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
});
