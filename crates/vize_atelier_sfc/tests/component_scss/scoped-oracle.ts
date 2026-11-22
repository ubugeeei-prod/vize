import assert from "node:assert/strict";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { compileStyle } = require("@vue/compiler-sfc");
const selectors = require("postcss-selector-parser");

type Rule = {
  selector: string;
  source: { start: { offset: number } };
  raws: { selector?: { raw: string } };
};
type ClassNode = { type: string; sourceIndex: number; value: string; raws?: { value?: string } };
export const scopeId = "data-v-oracle";

// The bounded scoped emitter retains authored whitespace, including spacing
// around commas. Compare Vue's scoped selectors and locate their insertions
// on the original CSS using PostCSS's independent position information.
export function scopedBlock(body: string, filename: string) {
  function rules(source: string, scoped: boolean): Rule[] {
    const result = compileStyle({ source, filename, scoped, id: scopeId, trim: false });
    assert.deepEqual(result.errors, []);
    const rules: Rule[] = [];
    result.rawResult.root.walkRules((rule: Rule) => {
      rules.push(rule);
    });
    return rules;
  }
  const insertions: number[] = [];
  for (const rule of rules(body, false)) {
    const raw = rule.raws.selector?.raw ?? rule.selector;
    for (const selector of selectors().astSync(raw).nodes) {
      const classes = selector.nodes.filter((node: ClassNode) => node.type === "class");
      assert.equal(classes.length, 1);
      const name: ClassNode = classes[0];
      insertions.push(
        rule.source.start.offset + name.sourceIndex + 1 + (name.raws?.value ?? name.value).length,
      );
    }
  }
  assert.ok(insertions.length);
  let css = "";
  let cursor = 0;
  const links: [number, number][] = [];
  for (const end of [...insertions, body.length]) {
    assert.ok(end >= cursor && end <= body.length);
    for (let pos = cursor; pos < end;) {
      links.push([css.length + pos - cursor, pos]);
      const lf = body.indexOf("\n", pos);
      if (lf < 0 || lf >= end) break;
      pos = lf + 1;
    }
    css += body.slice(cursor, end);
    if (end !== body.length) css += `[${scopeId}]`;
    cursor = end;
  }
  const canonical = (rule: Rule) => selectors().processSync(rule.selector, { lossless: false });
  assert.deepEqual(rules(css, false).map(canonical), rules(body, true).map(canonical));
  return { css, links };
}
