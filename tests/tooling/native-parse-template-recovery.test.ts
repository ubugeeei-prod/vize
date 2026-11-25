import assert from "node:assert/strict";
import test from "node:test";
import { compile, parseTemplate } from "../../npm/native/index.js";

const selfClosing = "<Transition><div /></Transition>";

test("parseTemplate accepts a self-closing non-void element in an SFC template", () => {
  const expected = {
    type: "ROOT",
    children: [
      {
        type: "ELEMENT",
        tag: "Transition",
        tagType: "Component",
        props: 0,
        children: 1,
        isSelfClosing: false,
      },
    ],
    comments: [],
    helpers: [],
    components: [],
    directives: [],
  };
  // The default and an explicit `standard` template syntax accept it, as `compile` does.
  assert.deepEqual(parseTemplate(selfClosing), expected);
  assert.deepEqual(parseTemplate(selfClosing, { templateSyntax: "standard" }), expected);
  assert.equal(typeof compile(selfClosing, { templateSyntax: "standard" }).code, "string");
});

test("the strict template syntax is how a project opts into the self-closing error", () => {
  for (const run of [
    () => parseTemplate(selfClosing, { templateSyntax: "strict" }),
    () => compile(selfClosing, { templateSyntax: "strict" }),
  ]) {
    assert.throws(run, /^Error: Parse errors: .*UnexpectedSolidusInTag/s);
  }
});

test("parseTemplate still throws for a template syntax error", () => {
  assert.throws(() => parseTemplate("<div /><span>"), /^Error: Parse errors: .*MissingEndTag/s);
});
