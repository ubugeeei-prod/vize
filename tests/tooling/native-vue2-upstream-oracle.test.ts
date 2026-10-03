import assert from "node:assert/strict";
import test from "node:test";
import { compiler, parser, plain } from "./support/vue2-pinned-oracle.ts";

const cases = [
  ["message | upper", '_f("upper")(message)'],
  ["value | add(2) | wrap('雪', '後')", "_f(\"wrap\")(_f(\"add\")(value,2),'雪', '後')"],
  ["left || right | upper", '_f("upper")(left || right)'],
  ["(left | right) | number", '_f("number")((left | right))'],
  ["'a|b' | upper", "_f(\"upper\")('a|b')"],
  ['"a|b" | upper', '_f("upper")("a|b")'],
  ["`a|${value}` | upper", '_f("upper")(`a|${value}`)'],
  ["/a|b/.test(value) | number", '_f("number")(/a|b/.test(value))'],
  ["value / 2 | number", '_f("number")(value / 2)'],
  ["list[index | mask] | number", '_f("number")(list[index | mask])'],
  ["({a: value | mask}) | json", '_f("json")(({a: value | mask}))'],
  [
    "value | pick(fn(1, 2), [3, 4], {a: 'x,y'}, /a,b/, `x,y`)",
    "_f(\"pick\")(value,fn(1, 2), [3, 4], {a: 'x,y'}, /a,b/, `x,y`)",
  ],
  ["雪 | 日本語", '_f("日本語")(雪)'],
  ["value | with-dash", '_f("with-dash")(value)'],
  ["value | upper()", '_f("upper")(value)'],
  ["value | upper ()", '_f("upper ")(value)'],
  ["value", "value"],
  ["value\r\n | upper", '_f("upper")(value)'],
] as const;

test("complete actual Vue2 parseText/filter values match whole independent goldens", () => {
  for (const [content, binding] of cases) {
    assert.equal(parser.parseFilters(content), binding, content);
    const text = `前 {{ ${content} }} 後`;
    const expected = {
      expression: `"前 "+_s(${binding})+" 後"`,
      tokens: ["前 ", { "@binding": binding }, " 後"],
    };
    assert.deepEqual(plain(parser.parseText(text)), expected, content);
    const compiled = compiler.compile(`<div>${text}</div>`);
    assert.deepEqual(plain(compiled.errors), [], content);
    assert.deepEqual(
      plain(
        compiled.ast.children.map(({ type, expression, tokens }) => ({ type, expression, tokens })),
      ),
      [{ type: 2, ...expected }],
      content,
    );
  }
});

test("historical text framing and deferred families stay explicitly characterized", () => {
  for (const content of ["{{}}", "{{ x\r y }}", "{{ x\u2028y }}", "{{ x\u2029y }}", "{{ x"]) {
    assert.equal(plain(parser.parseText(content)), null, content);
  }
  assert.deepEqual(plain(parser.parseText("{{ x\r\n + y }}")), {
    expression: "_s(x\r\n + y)",
    tokens: [{ "@binding": "x\r\n + y" }],
  });
  assert.deepEqual(plain(parser.parseText("{{{ x }}}")), {
    expression: '_s({ x)+"}"',
    tokens: [{ "@binding": "{ x" }, "}"],
  });
  assert.deepEqual(plain(parser.parseText("{{ }}")), {
    expression: "_s()",
    tokens: [{ "@binding": "" }],
  });
  assert.equal(parser.parseFilters("x | wrap(...values)"), '_f("wrap")(x,...values)');
  assert.equal(parser.parseFilters("x | wrap(1,)"), '_f("wrap")(x,1,)');
  assert.equal(parser.parseFilters("/[a|b]/.test(x) | number"), '_f("number")(/[a|b]/.test(x))');
  const compiled = compiler.compile("<div>&#123;&#123; x &#125;&#125;</div>");
  assert.equal(compiled.ast.children[0].expression, "_s(x)");
  for (const name of ["v-pre", "v-pre.foo", "v-pre:arg", "@pre", "v-previous"]) {
    const result = compiler.compile(`<div ${name}>{{ x | upper }}</div>`);
    assert.equal(result.ast.children[0].type, name === "v-pre" ? 3 : 2, name);
  }
});
