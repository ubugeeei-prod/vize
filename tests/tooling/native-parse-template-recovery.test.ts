import assert from "node:assert/strict";
import test from "node:test";
import { parseTemplate } from "../../npm/native/index.js";

test("parseTemplate accepts a self-closing non-void element in an SFC template", () => {
  assert.deepEqual(parseTemplate("<Transition><div /></Transition>"), {
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
  });
});

test("parseTemplate still throws for a template syntax error", () => {
  assert.throws(() => parseTemplate("<div /><span>"), /^Error: Parse errors: .*MissingEndTag/s);
});
