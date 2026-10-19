import assert from "node:assert/strict";
import test from "node:test";

import { stampScopeAttribute } from "./art-style.js";

const scope = "data-v-12345678";

void test("stampScopeAttribute keeps `>` inside quoted attribute values (#9752)", () => {
  assert.equal(
    stampScopeAttribute(`<PressButton @press="() => {}" />`, scope),
    `<PressButton @press="() => {}" ${scope} />`,
  );
  assert.equal(
    stampScopeAttribute(
      `<div :class="{ on: a > b }" title='x>y'><i v-if="a < b">t</i></div>`,
      scope,
    ),
    `<div :class="{ on: a > b }" title='x>y' ${scope}><i v-if="a < b" ${scope}>t</i></div>`,
  );
});

void test("stampScopeAttribute leaves comments, closing tags and interpolations alone", () => {
  assert.equal(
    stampScopeAttribute(`<!-- <b> --><p>{{ a<b ? 1 : 2 }}</p>`, scope),
    `<!-- <b> --><p ${scope}>{{ a<b ? 1 : 2 }}</p>`,
  );
});

void test("stampScopeAttribute stamps each element once", () => {
  assert.equal(
    stampScopeAttribute(`<span ${scope}>x</span><br/><img\n  src="a.png"\n/>`, scope),
    `<span ${scope}>x</span><br ${scope} /><img\n  src="a.png" ${scope} />`,
  );
  assert.equal(
    stampScopeAttribute(`<a title="${scope}">x</a>`, scope),
    `<a title="${scope}" ${scope}>x</a>`,
  );
});
