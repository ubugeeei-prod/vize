import assert from "node:assert/strict";

import { canonical, identity, jsonBytes } from "./lint-preset-source-successors.ts";

const ruleMapPath = "tests/_fixtures/patina-eslint-vue-rule-map.json";
const handlerStyle = "vue/v-on-handler-style";

// #12336 records the Real Project Matrix's eslint-plugin-vue default options
// for vue/v-on-handler-style. Remove exactly that field before older ledgers
// replay; any later rule-map edit stays fatal until it has its own successor.
export function inverseMatrixRuleOptions12336(name: string, body: Buffer): Buffer {
  if (name !== ruleMapPath) return body;
  identity(body, {
    bytes: 56071,
    sha256: "0d218a073ef430f92fb55996f772166c6449fcdd0e1d0a67963a7c9148feed6b",
  });
  const value = canonical(body);
  const overridden = Object.entries(value.entries).filter(
    ([, entry]) => (entry as { matrixRuleOptions?: unknown }).matrixRuleOptions !== undefined,
  );
  assert.deepEqual(
    overridden.map(([rule]) => rule),
    [handlerStyle],
  );
  delete value.entries[handlerStyle].matrixRuleOptions;
  const predecessor = jsonBytes(value);
  identity(predecessor, {
    bytes: 55313,
    sha256: "9db2b7af4e34df904ac5d53727d9e80726a9f07b42c0c618d61aeb578a98f15e",
  });
  return predecessor;
}
