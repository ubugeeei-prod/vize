import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { publicNative } from "./installed.ts";

const [version, output] = process.argv.slice(2);
assert.ok(version && output, "published version and new receipt path are required");
const { native, packages, loaded } = publicNative(version);
const pairs = [
  ['<div><div is="vue:my-thing">x</div></div>', "<div><my-thing>x</my-thing></div>"],
  [
    '<div is="vue:my-thing" title="a" :count="count">x</div>',
    '<my-thing title="a" :count="count">x</my-thing>',
  ],
  ['<div is="v&#117;e:my-thing">x</div>', "<my-thing>x</my-thing>"],
];
const passed = [];
for (const ssr of [false, true]) {
  for (const [original, equivalent] of pairs) {
    const options = { mode: "function", ssr, hoistStatic: false };
    const actual = native.compile(original, options);
    const named = native.compile(equivalent, options);
    assert.deepEqual(
      { code: actual.code, preamble: actual.preamble, helpers: actual.helpers },
      { code: named.code, preamble: named.preamble, helpers: named.helpers },
    );
    assert.match(actual.code + actual.preamble, /resolveComponent/u);
    passed.push({ backend: ssr ? "SSR" : "DOM", original, equivalent });
  }
  const original = '<div is="my-thing">x</div>';
  const plain = native.compile(original, { mode: "function", ssr });
  assert.doesNotMatch(plain.code + plain.preamble, /resolveComponent/u);
  assert.match(plain.code + plain.preamble, /my-thing/u);
  passed.push({ backend: ssr ? "SSR" : "DOM", original, ordinaryIsRemainsNative: true });
}
const receipt = { issue: 8328, version, packages, loaded, passed, success: true };
writeFileSync(output, JSON.stringify(receipt, null, 2) + "\n", { flag: "wx" });
console.log(JSON.stringify(receipt));
