import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { contentMapperHelperAbi } from "../../npm/cli/scripts/content-mapper-helper-abi.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const source = (file: string) => readFileSync(path.join(root, file), "utf8");

void test("the native ABI snapshot qualifies all conditional entries and private dependencies", () => {
  const [file, snapshot] = contentMapperHelperAbi(root, new Map());
  assert.equal(source(file), snapshot, "regenerate the native ABI with the SDK declarations");
});

for (const extension of ["ts", "mts", "cts"]) {
  void test(`a missing consumed export invalidates the .d.${extension} entry`, () => {
    const file = `npm/cli/virtual-ts.d.${extension}`;
    const changed = source(file).replace(
      "type $ComponentRawProps<",
      "type $MissingComponentRawProps<",
    );
    assert.notEqual(changed, source(file));
    assert.throws(
      () => contentMapperHelperAbi(root, new Map([[file, changed]])),
      /must export \$ComponentRawProps|invalid SDK declaration/u,
    );
  });
}

void test("private bindings participate in the same dependency contract", () => {
  const file = "npm/cli/virtual-ts-emits.d.ts";
  const changed = source(file).replace(
    "type $LooseEmitPropsFactory<",
    "type $MissingLooseEmitPropsFactory<",
  );
  assert.notEqual(changed, source(file));
  assert.throws(
    () => contentMapperHelperAbi(root, new Map([[file, changed]])),
    /invalid SDK declaration/u,
  );
});

void test("core imports must be public through every helper frontend", () => {
  const file = "npm/cli/generated/content-mapper-types.d.ts";
  const changed = source(file).replace("  $ComponentInput,\n", "");
  assert.notEqual(changed, source(file));
  assert.throws(
    () => contentMapperHelperAbi(root, new Map([[file, changed]])),
    /must export \$ComponentInput/u,
  );
});

void test("every relative import must have an installed declaration target", () => {
  const file = "npm/cli/virtual-ts.d.mts";
  const changed = source(file).replace("./virtual-ts-emits.js", "./absent-emit-dependency.js");
  assert.notEqual(changed, source(file));
  assert.throws(
    () => contentMapperHelperAbi(root, new Map([[file, changed]])),
    /missing SDK dependency/u,
  );
});

void test("ambient implicit exports are qualified by TypeScript rather than modifier text", () => {
  const pending = new Map(
    ["ts", "mts", "cts"].map((extension) => {
      const file = `npm/cli/virtual-ts.d.${extension}`;
      const changed = source(file).replace(
        "export type $ComponentRawProps<",
        "type $ComponentRawProps<",
      );
      assert.notEqual(changed, source(file));
      return [file, changed];
    }),
  );
  assert.doesNotThrow(() => contentMapperHelperAbi(root, pending));
});
