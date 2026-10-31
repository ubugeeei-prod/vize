import assert from "node:assert/strict";
import test from "node:test";
import { getAttributeHyphenationOptions } from "./attribute-options.ts";

await test("attribute options retain old style tuples and project independent literal ignore lists", () => {
  for (const style of ["always", "never"]) {
    assert.deepEqual(getAttributeHyphenationOptions([style]), { attributeHyphenation: style });
    const ignore = ["active", "/foo/", "FOO", "\u0085", "\u200b"];
    const source = [style, { ignore }];
    const before = JSON.stringify(source);
    const result = getAttributeHyphenationOptions(source);
    assert.deepEqual(result, { attributeHyphenation: style, attributeHyphenationIgnore: ignore });
    assert.notEqual(result?.attributeHyphenationIgnore, ignore);
    assert.equal(JSON.stringify(source), before);
    assert.deepEqual(getAttributeHyphenationOptions([style, {}]), {
      attributeHyphenation: style,
      attributeHyphenationIgnore: [],
    });
  }
});

await test("canonical objects pass through the shared CLI policy without mutation", () => {
  const source = { style: "never", ignore: ["active"] };
  const result = getAttributeHyphenationOptions([source]);
  assert.deepEqual(result, { ruleOptions: { "vue/attribute-hyphenation": source } });
  assert.notEqual(result?.ruleOptions?.["vue/attribute-hyphenation"], source);
  assert.deepEqual(getAttributeHyphenationOptions([{}]), {
    ruleOptions: { "vue/attribute-hyphenation": {} },
  });
});

await test("malformed or conflicting complete policies produce no partial native options", () => {
  for (const source of [
    ["always", { ignore: [""] }],
    ["always", { ignore: ["\ufeff"] }],
    ["always", { ignore: ["a:exit"] }],
    ["always", { ignore: ["a", "a"] }],
    ["always", { ignore: [1] }],
    ["always", { ignore: null }],
    ["always", { ignoreTags: [] }],
    [{ style: null }],
    [{ style: "sometimes" }],
    [{ ignore: [" "] }],
    ["always", {}, {}],
  ])
    assert.equal(getAttributeHyphenationOptions(source), undefined);
  assert.throws(
    () => getAttributeHyphenationOptions([{ ignore: ["active"] }, { ignore: [] }]),
    TypeError,
  );
});
