import assert from "node:assert/strict";

export function actualTestcase(xml: string, name: string, binary: string) {
  assert(!/<!DOCTYPE|<!ENTITY/i.test(xml), "no external XML authority");
  const cases = xml.match(/<testcase\b[^>]*(?:\/\s*>|>[\s\S]*?<\/testcase>)/g) ?? [];
  assert(cases.length > 0);
  const matching = cases.filter(testcase => {
    const opening = testcase.match(/^<testcase\b([^>]*)>/)!;
    assert(opening);
    const attributes = [...opening[1].matchAll(/\s([A-Za-z_][\w:.-]*)\s*=\s*(?:"([^"]*)"|'([^']*)')/g)];
    assert.equal(new Set(attributes.map(attribute => attribute[1])).size, attributes.length, "unique actual attributes");
    const values = new Map(attributes.map(attribute => [attribute[1], attribute[2] ?? attribute[3]]));
    return values.get("name") === name && values.get("classname") === binary;
  });
  assert.equal(matching.length, 1, "one exact opening-tag testcase identity");
  assert(!/<(?:failure|error|skipped)\b/.test(matching[0]), "case genuinely passed");
  return { testcase: matching[0], totalCases: cases.length };
}
