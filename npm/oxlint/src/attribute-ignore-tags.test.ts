import assert from "node:assert/strict";
import { test } from "node:test";
import { getAttributeHyphenationOptions } from "./attribute-options.ts";

test("literal tag policies preserve canonical and official tuple forms without mutation", () => {
  for (const ignoreTags of [[], ["MyWidget"], ["", " ", ":exit", "//", "/name", "/x/\n"]]) {
    const policy = { ignoreTags };
    const before = structuredClone(policy);
    const canonical = getAttributeHyphenationOptions([policy]);
    assert.deepEqual(canonical, { ruleOptions: { "vue/attribute-hyphenation": policy } });
    assert.notEqual(canonical?.ruleOptions?.["vue/attribute-hyphenation"], policy);
    assert.deepEqual(getAttributeHyphenationOptions(["never", policy]), {
      attributeHyphenation: "never",
      attributeHyphenationIgnore: [],
      attributeHyphenationIgnoreTags: ignoreTags,
    });
    assert.deepEqual(policy, before);
  }
});

test("regex-looking tag policies explicitly refuse rather than silently changing matching semantics", () => {
  for (const tag of ["/^My/", "/MyWidget/i", "/x/y", "/x/gg", "/[/"]) {
    for (const options of [[{ ignoreTags: [tag] }], ["always", { ignoreTags: [tag] }]])
      assert.throws(
        () => getAttributeHyphenationOptions(options),
        /literal tag names only; regular-expression policies are unsupported/,
      );
  }
  for (const ignoreTags of [null, [1], ["MyWidget", "MyWidget"]])
    assert.equal(getAttributeHyphenationOptions([{ ignoreTags }]), undefined);
  assert.deepEqual(getAttributeHyphenationOptions(["always", { ignoreTags: [] }]), {
    attributeHyphenation: "always",
    attributeHyphenationIgnore: [],
    attributeHyphenationIgnoreTags: [],
  });
});
