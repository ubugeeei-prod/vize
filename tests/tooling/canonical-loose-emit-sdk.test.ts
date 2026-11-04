import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { emitInferenceExports } from "../../npm/cli/scripts/emit-inference-exports.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const files = ["ts", "mts", "cts"].map((extension) => `npm/cli/virtual-ts.d.${extension}`);
const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");

test("all public emit factories exactly follow the canonical producer", async () => {
  const outputs = await emitInferenceExports(root, new Map());
  assert.deepEqual([...outputs.keys()], files);
  for (const [file, source] of outputs) {
    assert.equal(source, read(file));
    assert.equal(source.match(/^export type \$EmitPropsFactory</gmu)?.length, 1);
    for (const helper of ["$LooseEmitProps", "$LooseEmitPropsFactory", "$EmitIsWide"]) {
      assert.equal(source.includes(`export type ${helper}<`), false);
    }
    assert.equal(source.includes("$ListenerArgs"), false);
  }
});

test("unknown, partial, duplicate, foreign, and newly exported families are refused", async () => {
  const file = files[0]!;
  const original = read(file);
  const mutations = [
    original.replace("readonly __vizeLooseListener: []", "readonly __vizeLooseListener: unknown[]"),
    original.replace(/^type \$LooseEmitProps = .*;\n/gmu, ""),
    original + "\nexport type $EmitPropsFactory<C> = C;\n",
    original.replace(
      "type $NativeEmitPropsFactory<",
      "type ForeignFactory = true;\ntype $NativeEmitPropsFactory<",
    ),
    original.replace("type $LooseEmitProps =", "export type $LooseEmitProps ="),
  ];
  for (const source of mutations) {
    assert.notEqual(source, original);
    await assert.rejects(() => emitInferenceExports(root, new Map([[file, source]])));
    assert.equal(read(file), original, "rejected input cannot change SDK bytes");
  }
  const shared =
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt";
  const changed = read(shared).replace(
    "readonly __vizeLooseListener: []",
    "readonly __vizeLooseListener: unknown[]",
  );
  await assert.rejects(() => emitInferenceExports(root, new Map([[shared, changed]])));
});
